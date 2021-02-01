export function mark(selector, line_start, column_start, line_end, column_end, style) {
    console.log("adding mark in "+selector);

    var mark = window.editors[selector].markText(
      {line: line_start, ch: column_start},
      {line: line_end, ch: column_end},
      {css: style}
    );
    window.marks.push(mark);
    console.log(window.marks);
}