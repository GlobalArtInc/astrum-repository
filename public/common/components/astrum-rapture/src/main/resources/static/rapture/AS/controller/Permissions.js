
/*global Ext, AS*/

/**
 * Permissions management controller.
 *
 * Fires the following events on the controller:
 * @event loading Fired before the Permission store load begins (ASTRUM-52583).
 * @event changed Fired after permissions are successfully loaded and installed.
 * @event loaderror Fired when the Permission store load fails; receives the failed operation.
 *
 * @since 3.0
 */
Ext.define('AS.controller.Permissions', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.State',
    'AS.Permissions'
  ],

  stores: [
    'Permission'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      controller: {
        '#State': {
          userchanged: me.fetchPermissions
        }
      },
      store: {
        '#Permission': {
          load: me.firePermissionsChanged,
          update: me.onUpdate,
          remove: me.firePermissionsChanged
        }
      }
    });
  },

  /**
   * Prime initial set of permissions from state.
   *
   * @override
   */
  onLaunch: function () {
    var me = this;

    me.fetchPermissions();
  },

  /**
   * @private
   */
  onUpdate: function (store, record, operation) {
    if (operation === Ext.data.Model.COMMIT) {
      this.firePermissionsChanged();
    }
  },

  /**
   * @private
   */
  fetchPermissions: function () {
    var me = this;

    //<if debug>
    me.logDebug('Fetching permissions...');
    //</if>

    // Fire 'loading' event so UI can show appropriate state (ASTRUM-52583)
    me.fireEvent('loading');

    me.getStore('Permission').load({
      callback: function(records, operation, success) {
        if (!success) {
          //<if debug>
          me.logError('Failed to load permissions');
          //</if>
          me.fireEvent('loaderror', operation);
        }
        // Note: 'load' event fires firePermissionsChanged on success automatically
      }
    });
  },

  /**
   * @private
   */
  firePermissionsChanged: function () {
    var me = this;

    AS.Permissions.setPermissions(me.getPermissions());

    //<if debug>
    me.logDebug('Permissions changed; Firing event');
    //</if>

    me.fireEvent('changed', AS.Permissions);
  },

  /**
   * @private
   * @return {Object} permissions
   */
  getPermissions: function () {
    var store = this.getStore('Permission'),
        perms = {};

    store.clearFilter();
    store.each(function (rec) {
      perms[rec.get('id')] = rec.get('permitted');
    });

    return perms;
  }

});
