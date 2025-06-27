use crate::{database::DB, error::BotError};
use chrono::{DateTime, Utc};
use image::RgbImage;
use itertools::Itertools;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackendError;
use poise::{ChoiceParameter, CreateReply, command};
use rand::rng;
use rand_distr::Distribution;
use serenity::all::*;
use snafu::ResultExt;
use std::io::Cursor;

use super::super::Context;

pub mod command {

    use super::*;

    // 为了完整性，这里是一个扩展版本的命令，支持不同的图表类型
    #[command(slash_command, guild_only, owners_only)]
    pub async fn active_chart(
        ctx: Context<'_>,
        member: Member,
        #[description = "图表类型"] chart_type: Option<ChartType>,
        #[description = "是否为临时消息（仅自己可见）"] ephemeral: Option<bool>,
    ) -> Result<(), BotError> {
        let guild_id = ctx
            .guild_id()
            .expect("Guild ID should be present in a guild context");
        let user_id = member.user.id;
        let data = DB.actives().get(user_id, guild_id).await?;
        // filter out data in last 24 hours
        let data = data
            .into_iter()
            .filter(|&d| d >= Utc::now() - chrono::Duration::days(1))
            .collect_vec();

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
            ChartType::Bar => generate_activity_chart(&data, member.display_name()),
            ChartType::Timeline => generate_timeline_chart(&data, member.display_name()),
        };
        // 如果图表生成失败，返回错误信息
        let chart_buffer = match chart_buffer {
            Ok(buffer) => buffer,
            Err(e) => {
                ctx.send(
                    CreateReply::default()
                        .content(format!("生成图表失败: {e}"))
                        .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
        };
        let mut buffer = Vec::new();
        chart_buffer
            .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
            .whatever_context::<&str, BotError>("Failed to write chart image")?;
        let attachment = CreateAttachment::bytes(buffer, "activity_chart.png");

        let reply = CreateReply::default()
            .content(format!(
                "📊 **{}** 的活跃数据可视化 ({})\n总计发言: {} 次",
                member.display_name(),
                chart_type.name(),
                data.len()
            ))
            .attachment(attachment)
            .ephemeral(ephemeral.unwrap_or_default());
        ctx.send(reply).await?;
        Ok(())
    }
}

/// 按小时聚合数据
fn aggregate_by_hour(data: &[DateTime<Utc>]) -> [u32; 24] {
    let now = Utc::now();
    data.iter()
        .filter_map(|&ts| {
            let delta = now - ts;
            let num_hours = delta.num_hours();
            if (0..24).contains(&num_hours) {
                Some(num_hours)
            } else {
                None
            }
        })
        .fold([0; 24], |mut acc, hour| {
            acc[hour as usize] += 1;
            acc
        })
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
                format!("{username} 的每小时活跃度"),
                ("Noto Sans CJK SC", 30).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                -24i32..0i32,
                0i32..*hourly_data.iter().max().unwrap_or(&0) as i32,
            )?;

        chart
            .configure_mesh()
            .axis_desc_style(("Noto Sans CJK SC", 20).into_font())
            .x_desc("时间（小时前）")
            .y_desc("发言次数")
            .draw()?;

        // 绘制柱状图
        chart.draw_series(hourly_data.iter().enumerate().map(|(hour, &count)| {
            Rectangle::new(
                [(-(hour as i32) - 1, count as i32), (-(hour as i32), 0)],
                BLACK.stroke_width(2),
            )
        }))?;
        // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 1, y)], &BLUE));

        chart
            .configure_series_labels()
            .label_font(("Noto Sans CJK SC", 15).into_font())
            .draw()?;
        root.present()?;
    }
    // 将缓冲区转换为RGB图像
    let buffer =
        RgbImage::from_raw(WIDTH, HEIGHT, buffer).ok_or(DrawingAreaErrorKind::LayoutError)?;

    Ok(buffer)
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
                format!("{username} 的发言时间线"),
                ("Noto Sans CJK SC", 30).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .top_x_label_area_size(20)
            .build_cartesian_2d(-24f32..-0f32, -3f32..3f32)?;

        chart
            .configure_mesh()
            .axis_desc_style(("Noto Sans CJK SC", 20).into_font())
            .x_desc("时间（小时前）")
            .y_label_formatter(&|_| String::new())
            .disable_y_axis() // 隐藏Y轴标签
            .disable_mesh()
            .draw()?;

        // 绘制发言时间点
        let mut rng = rng();
        let normal = rand_distr::Normal::new(0.0, 1.0).unwrap();
        let now = Utc::now();
        chart.draw_series(data.iter().map(|timestamp| {
            let delta = now - *timestamp;
            let delta = -delta.num_seconds() as f32 / 3600.0; // 转换为小时
            let y_offset = normal.sample(&mut rng); // 添加一些随机偏移量
            Circle::new((delta, y_offset), 3, RED.filled())
        }))?;

        root.present()?;
    }
    // 将缓冲区转换为RGBA图像
    let buffer = RgbImage::from_raw(WIDTH, HEIGHT, buffer)
        .ok_or_else(|| DrawingAreaErrorKind::LayoutError)?;

    Ok(buffer)
}

/// 图表类型枚举
#[derive(ChoiceParameter, Default)]
pub enum ChartType {
    /// 时间线 - 显示具体发言时间点
    #[name = "时间线"]
    #[default]
    Timeline,
    /// 柱状图 - 按小时统计发言次数
    #[name = "柱状图"]
    Bar,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_activity_chart() {
        let data = vec![
            DateTime::parse_from_rfc3339("2023-10-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2023-10-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2023-10-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2023-10-01T13:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2023-10-01T14:15:00Z")
                .unwrap()
                .with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2023-10-01T14:15:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ];
        let username = "测试用户";
        let chart = generate_activity_chart(&data, username);
        assert!(chart.is_ok());
        // save the chart to a file for manual inspection
        let mut file = std::fs::File::create("test_activity_chart.png").unwrap();
        chart
            .unwrap()
            .write_to(&mut file, image::ImageFormat::Png)
            .unwrap();
        let chart = generate_timeline_chart(&data, username);
        assert!(chart.is_ok());
        let mut file = std::fs::File::create("test_timeline_chart.png").unwrap();
        chart
            .unwrap()
            .write_to(&mut file, image::ImageFormat::Png)
            .unwrap();
    }
}
