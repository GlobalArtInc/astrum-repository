'use strict';

// Reads the English texts of the React strings objects. The modules that pass an object to localize() are loaded in
// Node: TypeScript compiles them, React builds their JSX, every other package is a stub, and localize() records what
// it is given. The texts are then flattened the way localize() looks them up.

const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');

const LOCALIZE = /[\\/]interface[\\/]localize\.[jt]sx?$/;
const ASSET = /\.(s?css|less|svg|png|jpe?g|gif|woff2?|ttf|eot)$/;
const UNRESOLVED = '\u0000';
const MARKERS = Array.from({length: 6}, (_, index) => `{${index}}`);

function reactStrings(root, files, dictionary) {
  const resolve = (name) => require.resolve(name, {paths: [root, process.env.ASTRUM_I18N_MODULES].filter(Boolean)});
  const ts = require(resolve('typescript'));
  const React = require(resolve('react'));
  const stubs = new WeakSet();
  const recorded = [];

  function record(name, strings, scope) {
    recorded.push({name, strings, scope: scope || ''});
    return strings;
  }

  function stub(name) {
    const target = function stubbed() {};
    const proxy = new Proxy(target, {
      get(_, property) {
        if (property === Symbol.toPrimitive || property === 'toString' || property === 'valueOf') {
          return () => UNRESOLVED;
        }
        if (property === '__esModule') {
          return true;
        }
        if (property === 'localize') {
          return record;
        }
        return stub(`${name}.${String(property)}`);
      },
      apply() {
        return stub(`${name}()`);
      },
      construct() {
        return stub(`new ${name}`);
      }
    });
    stubs.add(proxy);
    return proxy;
  }

  const compile = (module, filename) => {
    const output = ts.transpileModule(fs.readFileSync(filename, 'utf8'), {
      fileName: filename,
      compilerOptions: {
        module: ts.ModuleKind.CommonJS,
        target: ts.ScriptTarget.ES2020,
        jsx: ts.JsxEmit.React,
        esModuleInterop: true,
        allowJs: true
      }
    });
    module._compile(output.outputText, filename);
  };

  const originalLoad = Module._load;
  const originalExtensions = {};
  for (const extension of ['.js', '.jsx', '.ts', '.tsx']) {
    originalExtensions[extension] = Module._extensions[extension];
    Module._extensions[extension] = compile;
  }
  Module._load = function load(request, parent, isMain) {
    if (request === 'react' || request.startsWith('react/')) {
      return require(resolve(request));
    }
    if (ASSET.test(request)) {
      return {};
    }
    if (/(^|\/)interface\/localize(\.[jt]sx?)?$/.test(request)) {
      return {__esModule: true, default: record};
    }
    if (!request.startsWith('.') && !path.isAbsolute(request)) {
      return stub(request);
    }
    if (LOCALIZE.test(Module._resolveFilename(request, parent, isMain))) {
      return {__esModule: true, default: record};
    }
    return originalLoad.call(this, request, parent, isMain);
  };

  try {
    for (const file of files) {
      require(path.join(root, file));
    }
  }
  finally {
    Module._load = originalLoad;
    for (const [extension, handler] of Object.entries(originalExtensions)) {
      if (handler) {
        Module._extensions[extension] = handler;
      }
      else {
        delete Module._extensions[extension];
      }
    }
    for (const name of Object.keys(require.cache)) {
      if (!name.includes(`${path.sep}node_modules${path.sep}`)) {
        delete require.cache[name];
      }
    }
  }

  const entries = {};
  const skipped = [];
  const own = recorded.filter((entry) => entry.name === dictionary);
  if (own.length === 0) {
    throw new Error(`No module of ${files.join(', ')} calls localize('${dictionary}', ...)`);
  }
  for (const {strings, scope} of own) {
    walk(strings, scope, new Set());
  }
  return {entries, skipped};

  function walk(node, prefix, visited) {
    if (visited.has(node)) {
      return;
    }
    visited.add(node);
    for (const [key, value] of Object.entries(node)) {
      const name = prefix ? `${prefix}.${key}` : key;
      if (stubs.has(value)) {
        continue;
      }
      if (typeof value === 'string') {
        if (!identifier(key, value)) {
          add(name, value);
        }
      }
      else if (React.isValidElement(value)) {
        add(name, serialize(value.props.children));
      }
      else if (typeof value === 'function') {
        add(name, templateOf(name, value));
      }
      else if (value && typeof value === 'object' && !isIcon(value)) {
        const prototype = Object.getPrototypeOf(value);
        if (prototype === Object.prototype || prototype === Array.prototype || prototype === null) {
          walk(value, name, visited);
        }
      }
    }
  }

  function add(name, text) {
    if (text === null) {
      return;
    }
    if (text.includes(UNRESOLVED) || /\bundefined\b|\[object /.test(text)) {
      skipped.push(`${name}: depends on a value only known at runtime`);
      return;
    }
    if (translatable(text)) {
      entries[name] = text;
    }
  }

  function templateOf(name, template) {
    const render = (args) => {
      const output = template(...args);
      if (typeof output === 'string') {
        return output;
      }
      if (React.isValidElement(output)) {
        return serialize(output.props.children);
      }
      throw new Error('returns neither a text nor an element');
    };
    try {
      const text = render(MARKERS);
      for (const value of [1, 0, '']) {
        if (render(MARKERS.map(() => value)) !== text.replace(/\{\d+\}/g, String(value))) {
          skipped.push(`${name}: its text depends on its arguments`);
          return null;
        }
      }
      return text;
    }
    catch (e) {
      skipped.push(`${name}: ${e.message}`);
      return null;
    }
  }

  function serialize(children, counter = {next: 0}) {
    let text = '';
    React.Children.forEach(children, (child) => {
      if (React.isValidElement(child)) {
        const index = counter.next++;
        const inner = child.props.children;
        text += inner === undefined || inner === null ? `<${index}/>` : `<${index}>${serialize(inner, counter)}</${index}>`;
      }
      else if (typeof child === 'string' || typeof child === 'number') {
        text += child;
      }
      else if (stubs.has(child)) {
        text += UNRESOLVED;
      }
    });
    return text;
  }
}

const IDENTIFIER_KEYS = new Set(['id', 'format', 'property', 'value', 'key', 'type']);

function identifier(key, text) {
  if (/\s/.test(text)) {
    return false;
  }
  return /^[a-z0-9]+([._-][a-z0-9]+)+$/.test(text) ||
      /^[A-Za-z0-9]+(_[A-Za-z0-9]+)+$/.test(text) ||
      /^[a-z]+[A-Z][A-Za-z0-9]*$/.test(text) ||
      /^[A-Z][a-z0-9]+([A-Z][a-z0-9]+){2,}$/.test(text) ||
      /(Realm|Exception)$/.test(text) ||
      (text === text.toLowerCase() && key.toLowerCase() === text) ||
      IDENTIFIER_KEYS.has(key);
}

function isIcon(value) {
  return typeof value.iconName === 'string' && Array.isArray(value.icon);
}

function translatable(text) {
  const plain = text.replace(/<\/?\d+\/?>|\{\d+\}/g, '');
  if (!/[A-Za-z]/.test(plain)) {
    return false;
  }
  return !/^(https?:\/\/|mailto:|\/|#)\S*$/.test(text.trim());
}

module.exports = {reactStrings};
