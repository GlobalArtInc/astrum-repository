
/*global Ext, NX*/

/**
 * Anonymous Security Settings controller.
 *
 * @since 3.21
 */
Ext.define('NX.coreui.controller.react.ReactViewController', {
  extend: 'NX.app.Controller',

  views: [
    'react.MainContainer'
  ],

  refs: [
    {
      ref: 'reactMainContainer',
      selector: 'nx-coreui-react-main-container'
    },
    {
      ref: 'breadcrumb',
      selector: 'nx-breadcrumb'
    },
  ],

  listen: {
    controller: {
      '#Refresh': {
        refresh: 'refresh'
      },
      '#Menu': {
        refresh: 'refresh'
      },
      '#State': {
        changed: function() {
          if (Ext.dom.Query.select('div.as-usage-center')[0]) {
            this.refresh();
          }
        }
      },
    },
    component: {
      'nx-coreui-react-main-container': {
        render: function() {
          this.getBreadcrumb().hide();
        },
        destroy: function() {
          this.getBreadcrumb().show();
        }
      }
    }
  },

  refresh: function() {
    if (this.getReactMainContainer()) {
      this.getReactMainContainer().refresh();
    }
  }
});
