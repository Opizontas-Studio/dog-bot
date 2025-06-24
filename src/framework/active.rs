use crate::database::DB;
use crate::error::BotError;
use chrono::{DateTime, Timelike, Utc};
use image::RgbImage;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackendError;
use poise::{ChoiceParameter, CreateReply, command};
use serenity::all::*;
use std::collections::HashMap;

use super::Context;

pub mod command {

    use std::io::Cursor;

    use super::*;

    // 为了完整性，这里是一个扩展版本的命令，支持不同的图表类型
    #[command(slash_command, guild_only, owners_only)]
    pub async fn active_chart(
        ctx: Context<'_>,
        member: Member,
        #[description = "图表类型: bar(柱状图), timeline(时间线), heatmap(热力图)"]
        chart_type: Option<ChartType>,
    ) -> Result<(), BotError> {
        let guild_id = ctx
            .guild_id()
            .expect("Guild ID should be present in a guild context");
        let user_id = member.user.id;
        let data = DB.actives().get(user_id, guild_id)?;

        if data.is_empty() {
            ctx.send(
                CreateReply::default()
                    .content("该用户今天还没有发言记录。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
        // 如果没有指定图表类型，则默认使用柱状图
        let chart_type = chart_type.unwrap_or_default();
        let chart_buffer = match chart_type {
            ChartType::Bar => generate_activity_chart(&data, &member.display_name()),
            ChartType::Timeline => generate_timeline_chart(&data, &member.display_name()),
            ChartType::Heatmap => generate_heatmap_chart(&data, &member.display_name()),
        };
        // 如果图表生成失败，返回错误信息
        let chart_buffer = match chart_buffer {
            Ok(buffer) => buffer,
            Err(e) => {
                ctx.send(
                    CreateReply::default()
                        .content(format!("生成图表失败: {}", e))
                        .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
        };
        let mut buffer = Vec::new();
        chart_buffer
            .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| BotError::GenericError {
                message: format!("图表写入缓冲区失败: {}", e),
                source: None,
            })?;
        let attachment = CreateAttachment::bytes(buffer, "activity_chart.png");

        let reply = CreateReply::default()
            .content(format!(
                "📊 **{}** 的活跃数据可视化 ({})\n总计发言: {} 次",
                member.display_name(),
                match chart_type {
                    ChartType::Bar => "柱状图",
                    ChartType::Timeline => "时间线",
                    ChartType::Heatmap => "热力图",
                },
                data.len()
            ))
            .attachment(attachment);

        ctx.send(reply).await?;
        Ok(())
    }
}

/// 生成活跃数据可视化图表
fn generate_activity_chart(
    data: &[DateTime<Utc>],
    username: &str,
) -> Result<RgbImage, DrawingAreaErrorKind<BitMapBackendError>> {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    let mut buffer = vec![0; (WIDTH * HEIGHT * 4) as usize]; // 创建一个800x600的RGBA缓冲区

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (WIDTH, HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        // 按小时统计发言次数
        let hourly_data = aggregate_by_hour(data);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                &format!("{} 的每小时活跃度", username),
                ("Noto Sans CJK SC", 30).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0u32..23u32,
                0u32..hourly_data.values().max().unwrap_or(&0) + 1,
            )?;

        chart
            .configure_mesh()
            .x_desc("小时 (UTC)")
            .y_desc("发言次数")
            .draw()?;

        // 绘制柱状图
        chart
            .draw_series(
                hourly_data.iter().map(|(&hour, &count)| {
                    Rectangle::new([(hour, 0), (hour, count)], BLUE.filled())
                }),
            )?
            .label("发言次数")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

        chart.configure_series_labels().draw()?;
        root.present()?;
    }
    // 将缓冲区转换为RGB图像
    let buffer = RgbImage::from_raw(WIDTH, HEIGHT, buffer)
        .ok_or_else(|| DrawingAreaErrorKind::LayoutError)?;

    Ok(buffer)
}

/// 按小时聚合数据
fn aggregate_by_hour(data: &[DateTime<Utc>]) -> HashMap<u32, u32> {
    let mut hourly_count = HashMap::new();

    for timestamp in data {
        let hour = timestamp.hour();
        *hourly_count.entry(hour).or_insert(0) += 1;
    }

    // 确保所有小时都有数据（即使是0）
    for hour in 0..24 {
        hourly_count.entry(hour).or_insert(0);
    }

    hourly_count
}

/// 生成时间线图表（显示具体的发言时间点）
fn generate_timeline_chart(
    data: &[DateTime<Utc>],
    username: &str,
) -> Result<RgbImage, DrawingAreaErrorKind<BitMapBackendError>> {
    const WIDTH: u32 = 1000;
    const HEIGHT: u32 = 400;
    let mut buffer = vec![0; (WIDTH * HEIGHT * 4) as usize]; // 创建一个1000x400的RGBA缓冲区

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (WIDTH, HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                &format!("{} 的发言时间线", username),
                ("Noto Sans CJK SC", 30).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0f32..24f32, -1f32..1f32)?;

        chart
            .configure_mesh()
            .x_desc("时间 (UTC)")
            .y_label_formatter(&|_| String::new()) // 隐藏Y轴标签
            .draw()?;

        // 绘制发言时间点
        chart.draw_series(data.iter().enumerate().map(|(i, timestamp)| {
            let hour = timestamp.hour() as f32 + (timestamp.minute() as f32 / 60.0);
            let y_offset = (i % 3) as f32 * 0.3 - 0.3; // 错开显示避免重叠
            Circle::new((hour, y_offset), 3, RED.filled())
        }))?;

        root.present()?;
    }
    // 将缓冲区转换为RGBA图像
    let buffer = RgbImage::from_raw(WIDTH, HEIGHT, buffer)
        .ok_or_else(|| DrawingAreaErrorKind::LayoutError)?;

    Ok(buffer)
}

/// 生成热力图风格的图表
fn generate_heatmap_chart(
    data: &[DateTime<Utc>],
    username: &str,
) -> Result<RgbImage, DrawingAreaErrorKind<BitMapBackendError>> {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 200;
    let mut buffer = vec![0; (WIDTH * HEIGHT * 4) as usize]; // 创建一个800x200的RGBA缓冲区

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (WIDTH, HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        let hourly_data = aggregate_by_hour(data);
        let max_count = *hourly_data.values().max().unwrap_or(&1) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                &format!("{} 的活跃热力图", username),
                ("Noto Sans CJK SC", 20).into_font(),
            )
            .margin(20)
            .x_label_area_size(30)
            .build_cartesian_2d(0u32..23u32, 0u32..0u32)?;

        chart.configure_mesh().x_desc("小时 (UTC)").draw()?;

        // 绘制热力图
        for hour in 0..24 {
            let count = *hourly_data.get(&hour).unwrap_or(&0) as f64;
            let intensity = if max_count > 0.0 {
                count / max_count
            } else {
                0.0
            };

            // 根据强度计算颜色
            let color = if intensity == 0.0 {
                RGBColor(240, 240, 240)
            } else {
                RGBColor(
                    (255.0 * (1.0 - intensity * 0.7)) as u8,
                    (255.0 * (1.0 - intensity * 0.8)) as u8,
                    255,
                )
            };

            let rect = Rectangle::new([(hour, 0), (hour + 1, 1)], color.filled());
            chart.draw_series(std::iter::once(rect))?;
        }

        root.present()?;
    }
    // 将缓冲区转换为RGBA图像
    let buffer = RgbImage::from_raw(WIDTH, HEIGHT, buffer)
        .ok_or_else(|| DrawingAreaErrorKind::LayoutError)?;

    Ok(buffer)
}

/// 图表类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, ChoiceParameter)]
pub enum ChartType {
    /// 柱状图 - 按小时统计发言次数
    #[name = "柱状图"]
    Bar,
    /// 时间线 - 显示具体发言时间点
    #[name = "时间线"]
    Timeline,
    /// 热力图 - 用颜色表示活跃程度
    #[name = "热力图"]
    Heatmap,
}

impl Default for ChartType {
    fn default() -> Self {
        Self::Bar
    }
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Bar => write!(f, "bar"),
            ChartType::Timeline => write!(f, "timeline"),
            ChartType::Heatmap => write!(f, "heatmap"),
        }
    }
}

impl std::str::FromStr for ChartType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bar" | "柱状图" => Ok(ChartType::Bar),
            "timeline" | "时间线" => Ok(ChartType::Timeline),
            "heatmap" | "热力图" => Ok(ChartType::Heatmap),
            _ => Err(format!(
                "无效的图表类型: {}。支持的类型: bar, timeline, heatmap",
                s
            )),
        }
    }
}
