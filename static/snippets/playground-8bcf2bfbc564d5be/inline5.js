export function register_parse_error(selector, message, line_start, column_start, line_end, column_end) {
    window.editor_lints[selector].push({
        message: message,
        severity: "error",
        from: CodeMirror.Pos(line_start, column_start),
        to: CodeMirror.Pos(line_end, column_end),
    });

}