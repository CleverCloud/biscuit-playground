export function set_query_result(parent, result) {
    var element = document.querySelector(parent + " .query-result");
    element.innerText = result;
}