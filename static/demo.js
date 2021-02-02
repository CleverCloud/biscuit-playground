import init, {testBiscuit} from "./wasm.js"


function setup_editor(selector) {
  var editor = document.querySelector(selector);
  let cm = new CodeMirror.fromTextArea(editor, {
    mode: 'biscuit',
    autoCloseTags: true,
    lineNumbers: true
  });

  function updateTextArea() {
    // somehow save() does not work
    //cm.save();
    var text = cm.getValue();

    let editor = document.querySelector(selector);
    editor.innerText = text;
    editor.value = text;

    const event = new Event('input');
    editor.dispatchEvent(event);
  }
  cm.on('change', updateTextArea);
  window.editors[selector] = cm;
}

function add_block(parent_sel, content) {
  let id = window.block_count[parent_sel];
  let block_list = document.querySelector(parent_sel + " .block-list");

  var element = document.createElement("li");
  element.class = "sub-container";

  var block_name;
  var should_display;
  if(id == 0) {
    block_name = "Authority block";
    should_display = "display: none";
  } else {
    block_name = "Block " + id;
    should_display = "";
  }

  element.innerHTML = `
    <div class="block block-${id}">
        <div>
            <span>
                <button onclick="delete_block('${parent_sel}', ${id})"
                    style="${should_display}"
                >-</button>
                <h4 style="display:inline">${ block_name }</h4>
            </span>
            <br />
        </div>

        <textarea
            class="code block-code ${ "block-code-" + id }"
            oninput="contentUpdate('${parent_sel}')"
        >${ content }</textarea>
    </div>`;

  block_list.appendChild(element);

  setup_editor(parent_sel + " .block-code-"+id);
  window.block_count[parent_sel] += 1;
}

window.add_block = add_block;
window.setup_editor = setup_editor;
window.block_count = [];
window.editors = {};
window.marks = [];

function delete_block(parent_sel, id) {
  let block = document.querySelector(parent_sel + " .block-"+id);
  let li = block.parentNode;
  li.parentNode.removeChild(li);
  window.block_count[parent_sel] -= 1;
  delete window.editors[parent_sel + ".block-"+id];
}

window.delete_block = delete_block;

function contentUpdate(parent_sel) {
  console.log("will call testBiscuit");
  testBiscuit(parent_sel)

}

window.contentUpdate = contentUpdate;

async function setup(parent_selector) {
  console.log("will call wasm.default()");
  await init()

  console.log("setting up for "+parent_selector);
  window.block_count[parent_selector] = 0;
  window.editors[parent_selector] = {};
}

window.setup = setup;
console.log("setup function defined");

export { setup }
