
/*global Ext, NX*/

/**
 * A {@link NX.util.condition.Condition} that is satisfied when user has a specified permission.
 *
 * @since 3.0
 */
Ext.define('NX.util.condition.IsPermitted', {
  extend: 'NX.util.condition.Condition',

  /**
   * @private
   * @property {String}
   */
  permission: undefined,

  /**
   * @override
   * @returns {NX.util.condition.IsPermitted}
   */
  bind: function () {
    var me = this,
        controller;

    if (!me.bounded) {
      controller = NX.getApplication().getController('Permissions');
      me.mon(controller, {
        changed: me.evaluate,
        scope: me
      });
      me.callParent();
      me.evaluate();
    }

    return me;
  },

  /**
   * @private
   */
  evaluate: function () {
    var me = this;

    if (me.bounded) {
      me.setSatisfied(NX.Permissions.check(me.permission));
    }
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ permission=' + this.permission + ' }';
  },

  /**
   * Sets permission and re-evaluate.
   *
   * @public
   * @param {String} permission
   */
  setPermission: function(permission) {
    this.permission = permission;
    this.evaluate();
  }

});
