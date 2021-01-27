export function clear_marks() {
    for(var i=0; i < window.marks.length; i=i+1) {
        console.log("clearing mark "+i);
        window.marks[i].clear();
    }
    window.marks = [];
}