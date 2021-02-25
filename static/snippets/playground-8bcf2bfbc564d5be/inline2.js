export function set_verifier_result(parent, error, world) {
    var element = document.querySelector(parent + " .verifier-result");
    element.innerText = error;
    var element = document.querySelector(parent + " .verifier-world");
    element.innerHTML = world;
}