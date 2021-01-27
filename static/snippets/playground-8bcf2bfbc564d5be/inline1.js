export function mark(id, line_start, column_start, line_end, column_end, style) {
    console.log("adding mark in "+id);

    var mark = window.editors[id].markText(
      {line: line_start, ch: column_start},
      {line: line_end, ch: column_end},
      {css: style}
    );
    window.marks.push(mark);
    console.log(window.marks);
}