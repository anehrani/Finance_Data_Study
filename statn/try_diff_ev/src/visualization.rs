//! Visualization module for plotting trading signals.

use crate::backtest::TradeStats;
use crate::signals::SignalResult;
use plotters::prelude::*;
use std::path::Path;

/// Visualise the price series together with BUY/SELL markers.
///
/// The function writes a PNG file to the specified output path.
/// BUY signals are drawn as green upward triangles, SELL as red circles,
/// and HOLD points are omitted for clarity.
///
/// # Arguments
/// * `result` - Signal result containing prices and signals
/// * `output_path` - Path where the chart PNG will be saved
pub fn visualise_signals<P: AsRef<Path>>(
    result: &SignalResult,
    stats: Option<&TradeStats>,
    output_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path.as_ref(), (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_price = result.prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = result.prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Price chart with BUY/SELL signals", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(60)
        .build_cartesian_2d(0usize..result.prices.len(), min_price..max_price)?
        .set_secondary_coord(
            0usize..result.prices.len(),
            stats.map_or(0.0..1.0, |s| {
                let min_w = s.budget_history.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_w = s.budget_history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                min_w..max_w
            }),
        );

    chart.configure_mesh().disable_mesh().draw()?;

    // Plot the price line.
    chart
        .draw_series(LineSeries::new(
            result.prices.iter().enumerate().map(|(i, p)| (i, *p)),
            &BLUE,
        ))?
        .label("Price")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Plot BUY markers (green upward triangles).
    chart.draw_series(
        result
            .signals
            .iter()
            .enumerate()
            .filter(|&(_, &s)| s == 1)
            .map(|(i, _)| {
                let price = result.prices[i];
                TriangleMarker::new((i, price), 8, ShapeStyle::from(&GREEN).filled())
            }),
    )?
    .label("BUY")
    .legend(|(x, y)| TriangleMarker::new((x, y), 8, ShapeStyle::from(&GREEN).filled()));

    // Plot SELL markers (red circles to distinguish from BUY).
    chart.draw_series(
        result
            .signals
            .iter()
            .enumerate()
            .filter(|&(_, &s)| s == -1)
            .map(|(i, _)| {
                let price = result.prices[i];
                Circle::new((i, price), 5, ShapeStyle::from(&RED).filled())
            }),
    )?
    .label("SELL")
    .legend(|(x, y)| Circle::new((x, y), 5, ShapeStyle::from(&RED).filled()));

    // Plot wealth curve if stats provided
    if let Some(s) = stats {
        chart
            .draw_secondary_series(LineSeries::new(
                s.budget_history.iter().enumerate().map(|(i, w)| (i, *w)),
                &MAGENTA,
            ))?
            .label("Wealth")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &MAGENTA));
    }

    chart.configure_series_labels().border_style(&BLACK).draw()?;
    Ok(())
}
