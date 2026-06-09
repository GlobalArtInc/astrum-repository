
/*global Ext*/

/**
 * **{@link Ext.form.field.Checkbox}** override, that changes default width overridden in {@link Ext.form.field.Base}.
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.field.Checkbox', {
  override: 'Ext.form.field.Checkbox',

  width: undefined,

  initComponent: function () {
    var me = this;

    if (me.helpText && ! me.isHelpTextPlaced) {
      me.boxLabel = '<span class="nx-boxlabel">' + me.helpText + '</span>';
      me.isHelpTextPlaced = true;
    }

    me.callParent(arguments);
  }

});
