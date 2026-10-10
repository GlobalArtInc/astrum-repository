
/*global Ext*/

/**
 * **{@link Ext.form.FieldContainer}** override, that changes default label width.
 *
 * @since 3.0
 */
Ext.define('AS.ext.form.FieldContainer', {
  override: 'Ext.form.FieldContainer',

  labelAlign: 'top',
  labelStyle: 'font-weight: bold;',
  msgTarget: 'under',

  initComponent: function () {
    var me = this;

    if (me.helpText) {
      me.afterLabelTpl = '<span class="as-help-text-after-label">' + me.helpText + '</span>';
    }

    me.callParent(arguments);
  }

});
