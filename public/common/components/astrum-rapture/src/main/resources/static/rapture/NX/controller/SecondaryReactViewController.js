
/*global Ext, NX*/

/**
 * Secondary Security Settings controller.
 *
 */
Ext.define('NX.controller.SecondaryReactViewController', {
  extend: 'NX.app.Controller',

  views: [
    'SecondaryContainer'
  ],

  refs: [
    {
      ref: 'secondaryContainer',
      selector: 'nx-secondary-container'
    }
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
          this.refresh();
        }
      },
      '#Permissions': {
        changed: function() {
          this.refresh();
        }
      }
    }
  },

  refresh: function() {
    if (this.getSecondaryContainer()) {
      this.getSecondaryContainer().refresh();
    }
  }
});
