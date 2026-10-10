
/*global Ext, AS, REACT_COMPONENTS, react, ReactDOM */

/**
 * React Main Container
 *
 * @since 3.21
 */
Ext.define('AS.coreui.view.react.MainContainer', {
  extend: 'Ext.Component',
  alias: 'widget.as-coreui-react-main-container',

  reactView: undefined,

  reactViewProps: null,

  scrollable: true,

  cls: 'as-react-main-container',

  listeners: {
    afterrender: 'initReactView',
    beforedestroy: 'destroyReactView'
  },

  initReactView: function() {
    const reactElement = react.createElement(this.reactView, this.reactViewProps, null);
    ReactDOM.render(reactElement, this.getEl().dom);
  },

  destroyReactView: function() {
    if (this.reactView && this.getEl() && this.getEl().dom) {
      ReactDOM.unmountComponentAtNode(this.getEl().dom);
    }
  },

  refresh: function() {
    this.destroyReactView();
    this.initReactView();
  }
});
