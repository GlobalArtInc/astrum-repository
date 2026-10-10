
/*global Ext, AS*/

/**
 * Anonymous Security Settings controller.
 *
 * @since 3.21
 */
Ext.define('AS.coreui.controller.react.ReactViewController', {
  extend: 'AS.app.Controller',

  views: [
    'react.MainContainer'
  ],

  refs: [
    {
      ref: 'reactMainContainer',
      selector: 'as-coreui-react-main-container'
    },
    {
      ref: 'breadcrumb',
      selector: 'as-breadcrumb'
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
      'as-coreui-react-main-container': {
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
