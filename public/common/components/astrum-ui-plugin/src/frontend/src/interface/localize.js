import React from 'react';

const TOKEN = /<(\d+)\/>|<(\d+)>|<\/(\d+)>|\{(\d+)\}/g;

/**
 * Replaces the English texts of a strings object with their translations when the page is shown in another language.
 *
 * The translations come with the page as `window.AS_I18N.dictionaries[dictionary]`, keyed by the dotted path of each
 * text in the object, after the optional `scope` (see tools/i18n/README.md):
 * - a string is replaced by its translation;
 * - a function returning a string, e.g. `(name) => \`Edit ${name}\``, by one filling `{0}`, `{1}`... of the translation
 *   with its arguments;
 * - a JSX element, or a function returning one, by the same element around the translated text, in which `<0>...</0>`
 *   and `<1/>` stand for the element's descendants in document order.
 *
 * Texts without a translation, and translations that do not fit their text, stay English. The object is changed in
 * place, so modules that import a part of it directly see the translations too, and returned.
 */
export default function localize(dictionary, strings, scope) {
  const translations = dictionaryOf(dictionary);
  if (translations && strings && typeof strings === 'object') {
    translate(strings, scope || '', translations, new Set());
  }
  return strings;
}

function dictionaryOf(name) {
  const i18n = typeof window === 'undefined' ? undefined : window.AS_I18N;
  return (i18n && i18n.dictionaries && i18n.dictionaries[name]) || null;
}

function translate(node, prefix, translations, visited) {
  if (visited.has(node) || Object.isFrozen(node)) {
    return;
  }
  visited.add(node);
  Object.keys(node).forEach((key) => {
    const path = prefix ? `${prefix}.${key}` : key;
    const value = node[key];
    const translation = translations[path];
    if (isContainer(value)) {
      translate(value, path, translations, visited);
    }
    else if (typeof translation === 'string' && translation !== '') {
      if (typeof value === 'string') {
        node[key] = translation;
      }
      else if (React.isValidElement(value)) {
        node[key] = rebuild(value, translation, []);
      }
      else if (typeof value === 'function') {
        node[key] = template(value, translation);
      }
    }
  });
}

function isContainer(value) {
  if (!value || typeof value !== 'object' || React.isValidElement(value)) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === Array.prototype || prototype === null;
}

function template(original, translation) {
  return function localized(...args) {
    const text = original(...args);
    if (typeof text === 'string') {
      return args.some(React.isValidElement) ? text : fill(translation, args);
    }
    if (React.isValidElement(text)) {
      return rebuild(text, translation, args);
    }
    return text;
  };
}

function fill(translation, args) {
  return translation.replace(/\{(\d+)\}/g, (match, index) => (index < args.length ? String(args[index]) : match));
}

function rebuild(element, translation, args) {
  try {
    const elements = [];
    collect(element.props.children, elements);
    const children = parse(translation, elements, args);
    return children ? React.cloneElement(element, undefined, ...children) : element;
  }
  catch (e) {
    return element;
  }
}

function collect(children, elements) {
  React.Children.forEach(children, (child) => {
    if (React.isValidElement(child)) {
      elements.push(child);
      collect(child.props.children, elements);
    }
  });
}

function parse(translation, elements, args) {
  const stack = [{children: []}];
  const top = () => stack[stack.length - 1];
  let last = 0;
  let match;
  TOKEN.lastIndex = 0;
  while ((match = TOKEN.exec(translation)) !== null) {
    if (match.index > last) {
      top().children.push(translation.slice(last, match.index));
    }
    last = TOKEN.lastIndex;
    const [, single, open, close, arg] = match;
    if (single !== undefined) {
      if (!elements[single]) {
        return null;
      }
      top().children.push(elements[single]);
    }
    else if (open !== undefined) {
      if (!elements[open]) {
        return null;
      }
      stack.push({index: Number(open), children: []});
    }
    else if (close !== undefined) {
      const node = stack.pop();
      if (stack.length === 0 || node.index !== Number(close)) {
        return null;
      }
      top().children.push(React.cloneElement(elements[node.index], undefined, ...node.children));
    }
    else {
      top().children.push(Number(arg) < args.length ? args[arg] : match[0]);
    }
  }
  if (last < translation.length) {
    top().children.push(translation.slice(last));
  }
  return stack.length === 1 ? stack[0].children : null;
}
