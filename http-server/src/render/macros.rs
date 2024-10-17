#[macro_export]
macro_rules! node {
    ($f:ident, $tag:expr, $attrs:expr, $content:block) => {
        $f.write_str(concat!("<", $tag, " ", $attrs, ">"))?;
        $content
        $f.write_str(concat!("</", $tag, ">"))?;
    };
    ($f:ident, $tag:expr, $content:block) => {
        $f.write_str(concat!("<", $tag, ">"))?;
        $content
        $f.write_str(concat!("</", $tag, ">"))?;
    };
}
