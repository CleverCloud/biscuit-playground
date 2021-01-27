export function set_verifier_result(error, world) {
    var element = document.getElementById("verifier-result");
    element.innerText = error;
    var element = document.getElementById("verifier-world");
    element.innerText = world;
}