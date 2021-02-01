export function set_token_content(parent, content) {
    var element = document.querySelector(parent+" .token-content");
    element.innerText = content;
}