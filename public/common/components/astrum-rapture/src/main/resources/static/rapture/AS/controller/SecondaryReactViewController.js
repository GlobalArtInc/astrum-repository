
/*global Ext, AS*/

/**
 * Secondary Security Settings controller.
 *
 */
Ext.define('AS.controller.SecondaryReactViewController', {
  extend: 'AS.app.Controller',

  views: [
    'SecondaryContainer'
  ],

  refs: [
    {
      ref: 'secondaryContainer',
      selector: 'as-secondary-container'
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
