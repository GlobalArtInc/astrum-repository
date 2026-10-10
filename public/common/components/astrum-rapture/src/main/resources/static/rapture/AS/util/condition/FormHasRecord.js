
/*global Ext*/

/**
 * A {@link AS.util.condition.Condition} that is satisfied when a {@link AS.view.SettingsForm}, specified by its
 * selector, exists and has a record. Optionally, a function could be used to provide additional checking when form has
 * a record.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.FormHasRecord', {
  extend: 'AS.util.condition.Condition',

  /**
   * A form selector as specified by (@link Ext.ComponentQuery#query}.
   *
   * @cfg {String}
   */
  form: undefined,

  /**
   * An optional function to be called when form has a record to perform additional checks on the passed in model.
   *
   * @cfg {Function}
   */
  fn: undefined,

  /**
   * @override
   * @returns {AS.util.condition.FormHasRecord}
   */
  bind: function () {
    var me = this,
        queryResult = Ext.ComponentQuery.query(me.form),
        formCmp = queryResult && queryResult.length ? queryResult[0] : null;

    if (!me.bounded && formCmp) {
      formCmp.on({
        afterrender: me.evaluate,
        recordloaded: me.evaluate,
        destroy: me.evaluate,
        scope: me
      });
      me.callParent();
      me.evaluate(formCmp);
    }

    return me;
  },

  /**
   * @private
   */
  evaluate: function (form) {
    var me = this,
        satisfied = false,
        model;

    if (me.bounded) {
      if (Ext.isDefined(form) && form.isXType('form')) {
        model = form.getRecord();
        if (model) {
          satisfied = true;
          if (Ext.isFunction(me.fn)) {
            satisfied = me.fn(model) === true;
          }
        }
      }
      me.setSatisfied(satisfied);
    }
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ form=' + this.form + ' }';
  }

});