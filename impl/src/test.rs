/// Tests the equality of spans.
macro_rules! compare_span {
    ($span:expr, ($sl:expr, $sc:expr), ($el:expr, $ec:expr)) => {{
        let start = $span.start();
        let end = $span.end();
        assert_eq!(
            (start.line, start.column, end.line, end.column),
            ($sl, $sc, $el, $ec)
        );
    }}
}