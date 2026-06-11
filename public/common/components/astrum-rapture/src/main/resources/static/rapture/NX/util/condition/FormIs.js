
/*global Ext*/

/**
 * A {@link NX.util.condition.Condition} that is satisfied when a {@link NX.view.SettingsForm}, specified by its
 * selector, exists and has a record. Optionally, a function could be used to provide additional checking when form has
 * a record.
 *
 * @since 3.14
 */
Ext.define('NX.util.condition.FormIs', {
  extend: 'NX.util.condition.Condition',

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

  condition: undefined,

  formComponent: undefined,

  /**
   * @override
   * @returns {NX.util.condition.FormHasRecord}
   */
  bind: function () {
    if (Ext.isString(this.form)) {
      this.formComponent = Ext.first(this.form);
    }
    else {
      this.formComponent = this.form;
    }

    if (!this.bounded && this.formComponent) {
      this.formComponent.on({
        afterrender: this.evaluate,
        validitychange: this.evaluate,
        disable: this.evaluate,
        enable: this.evaluate,
        destroy: this.evaluate,
        scope: this
      });
      this.callParent();
      this.evaluate(this.formComponent);
    }

    return this;
  },

  /**
   * @private
   */
  evaluate: function () {
    if (this.bounded) {
      this.setSatisfied(this.condition(this.formComponent));
    }
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ form=#' + this.formComponent.getId() + ' }';
  }

});
