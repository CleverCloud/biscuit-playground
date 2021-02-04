import init, {testBiscuit} from "./wasm.js"


function setup_editor(selector) {
  var editor = document.querySelector(selector);

  window.editor_lints[selector] = [];
  /*window.editor_lints[selector] = [
    {
      message: "pouet",
      severity: "warning",
      from: CodeMirror.Pos(0, 1),
      to: CodeMirror.Pos(2, 4)
    }];*/

  function get_editor_lints() {
    var this_selector = selector;
    var lints = window.editor_lints[this_selector];
    return lints;
  }

  let cm = new CodeMirror.fromTextArea(editor, {
    mode: 'biscuit',
    autoCloseTags: true,
    lineNumbers: true,
    gutters: ["CodeMirror-lint-markers"],
    lintOnChange: false,
    lint: {
      getAnnotations: get_editor_lints,
    },
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
window.editor_lints = {};
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

  for (var key in window.editor_lints) {
    if (window.editor_lints.hasOwnProperty(key)) {
      delete window.editor_lints[key];
    }
  }

  testBiscuit(parent_sel)

  for (var key in window.editors) {
    if (window.editors.hasOwnProperty(key)) {
        var editor = window.editors[key];
        if(editor !== undefined) {
          // clear markers
          var state = editor.state.lint;
          if(state.hasGutter) editor.clearGutter("CodeMirror-lint-markers");
          for (var i = 0; i < state.marked.length; ++i) {
            state.marked[i].clear();
          }
          state.marked.length = 0;

          editor.setOption("lint", false);

          let this_selector = key;
          function get_editor_lints() {
            var lints = window.editor_lints[this_selector];
            console.log("lints for "+this_selector);
            console.debug(lints);
            return lints;
          }
          editor.setOption("lint", {
            getAnnotations: get_editor_lints,
          });
        }
    }
  }
}

window.contentUpdate = contentUpdate;

async function setup(parent_selector) {
  console.log("will call wasm.default()");
  await init()

  console.log("setting up for "+parent_selector);
  window.block_count[parent_selector] = 0;
  //window.editors[parent_selector] = {};
}

window.setup = setup;
console.log("setup function defined");

export { setup }
