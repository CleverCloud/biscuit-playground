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
            class="code ${ "block-code-" + id }"
            oninput="contentUpdate('${parent_sel}')"
        >${ content }</textarea>
    </div>`;

  block_list.appendChild(element);

  setup_editor(parent_sel + " .block-code-"+id);
  window.block_count[parent_sel] += 1;
}

window.add_block = add_block;
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


  function load() {
    add_block(parent_selector, `// this is a fact, the basic data used in Datalog
// you can see it as one line in the "right" table
// facts with #authority can only be in the first block
right(#authority, "/folder1/file1", #read)
right(#authority, "/folder1/file1", #write)
right(#authority, "/folder2/file1", #read)

// rules generate more facts from existing ones. You can
// read it as "generate can_read with the content $file
// if there exists a 'right' fact that matches"
can_read($file) <- right(#authority, $file, #read)

// this is a check. It will succeed if it finds matching
// facts. Otherwise, the token validation will fail
check if operation(#ambient, #read)
`);
    add_block(parent_selector,
`// to restrict rights, we add blocks with more checks
// checks and rules can also test expressions along with
// the presence of facts. To match, the expression must
// evaluate to true
// #ambient indicates facts provided by the verifier to
// evaluate the request, like which resource is accessed,
// or what is the current date
check if
  resource(#ambient, $file),
  $file.starts_with("/folder1/")
`);

    var verifier = document.querySelector(parent_selector + " .verifier-code");
    verifier.value = `// here we got a read request on /folder1/file1
resource(#ambient, "/folder1/file1")
operation(#ambient, #read)

// if this matches, the verification will succeed
allow if
  resource(#ambient, $file),
  operation(#ambient, $op),
  right(#authority, $file, $op)

// this catch-all policy will refuse the request
deny if true
`;

    setup_editor(parent_selector + " .verifier-code");
    const event = new Event('input');
    verifier.dispatchEvent(event);

  }


  load();
}

window.setup = setup;
console.log("setup function defined");

export { setup }
