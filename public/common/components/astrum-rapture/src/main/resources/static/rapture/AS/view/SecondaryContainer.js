
/*global Ext, AS, REACT_COMPONENTS, react, ReactDOM */

/**
 * React Secondary Container
 *
 */
Ext.define('AS.view.SecondaryContainer', {
  extend: 'Ext.Component',
  alias: 'widget.as-secondary-container',

  reactView: undefined,

  reactViewProps: null,

  root: null,

  scrollable: true,

  cls: 'astrum-secondary-container',

  listeners: {
    afterrender: 'initReactView',
    beforedestroy: 'destroyReactView'
  },

  initReactView: function() {
    const reactElement = react.createElement(this.reactView, this.reactViewProps, null);
    this.root = window.createRoot(this.getEl().dom);
    this.root.render(reactElement);
  },

  destroyReactView: function() {
    if (this.reactView && this.root) {
      this.root.unmount();
    }
  },

  refresh: function() {
    this.destroyReactView();
    this.initReactView();
  }
});
