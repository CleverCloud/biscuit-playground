
/* Example definition of a simple mode that understands a subset of
 * JavaScript:
 */

CodeMirror.defineSimpleMode("biscuit", {
  // The start state contains the rules that are initially used
  start: [
    {regex: /(allow if|deny if|check if|or|and|<-)\b/,
     token: "keyword"},
    {regex: /\/\/.*/, token: "comment"},
    {regex: /\/\*/, token: "comment", next: "comment"},

    // predicate name
    {regex: /([A-Za-z_][\w]*)/, token: "keyword", next: "terms"},

    {regex: /,/, token: "operator"},
    {regex: /"(?:[^\\]|\\.)*?(?:"|$)/, token: "string"},
    {regex: /\$[A-Za-z_][\w]*/, token: "variable"},
    {regex: /#[A-Za-z_][\w]*/, token: "symbol"},
    {regex: /true|false/, token: "atom"},
    // RFC 3339 date
    {regex: /(\d+)-(0[1-9]|1[012])-(0[1-9]|[12]\d|3[01])T([01]\d|2[0-3]):([0-5]\d):([0-5]\d|60)(\.\d+)?(([Zz])|([\+|\-]([01]\d|2[0-3]):([0-5]\d)))/, token: "atom" },
    {regex: /[-+]?\d+/i, token: "number"},


    // A next property will cause the mode to move to a different state
    {regex: /[-+\/*=<>!]+/, token: "operator"},
    {regex: /&&|\|\|/, token: "operator"},
    // indent and dedent properties guide autoindentation
    {regex: /[\{\[\(]/, indent: true},
    {regex: /[\}\]\)]/, dedent: true},
  ],
  // The multi-line comment state.
  comment: [
    {regex: /.*?\*\//, token: "comment", next: "start"},
    {regex: /.*/, token: "comment"}
  ],
  terms: [
    {regex: /,/, token: "operator"},
    // The regex matches the token, the token property contains the type
    {regex: /"(?:[^\\]|\\.)*?(?:"|$)/, token: "string"},
    {regex: /\$[A-Za-z_][\w]*/, token: "variable"},
    {regex: /#[A-Za-z_][\w]*/, token: "symbol"},
    {regex: /true|false/, token: "atom"},
    // RFC 3339 date
    {regex: /(\d+)-(0[1-9]|1[012])-(0[1-9]|[12]\d|3[01])T([01]\d|2[0-3]):([0-5]\d):([0-5]\d|60)(\.\d+)?(([Zz])|([\+|\-]([01]\d|2[0-3]):([0-5]\d)))/, token: "atom" },
    {regex: /[-+]?\d+/i, token: "number"},
    {regex: /\)/, next: "start"},
  ],
  // The meta property contains global information about the mode. It
  // can contain properties like lineComment, which are supported by
  // all modes, and also directives like dontIndentStates, which are
  // specific to simple modes.
  meta: {
    dontIndentStates: ["comment"],
    lineComment: "//"
  }
});

/*
let cm = new CodeMirror.fromTextArea(document.getElementById("editor"), {
  mode: 'biscuit',
  autoCloseTags: true,
  lineNumbers: true
});
*/
