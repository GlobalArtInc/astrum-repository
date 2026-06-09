
/*global Ext*/

/**
 * **{@link Ext.form.field.Base}** override, that changes default label width.
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.field.Base', {
  override: 'Ext.form.field.Base',

  plugins: {
    responsive:true
  },
  responsiveConfig: {
    'width <= 1366': {
      maxWidth: 600
    },
    'width <= 1600': {
      maxWidth: 800
    },
    'width > 1600' : {
      maxWidth: 1000
    }
  },
  width: '100%',

  labelAlign: 'top',
  labelStyle: 'font-weight: bold;',
  msgTarget: 'under',

  /**
   * @cfg {boolean} [hideIfUndefined=false]
   * If field should auto hide in case it has no value. Functionality applies only for read only field.
   */
  hideIfUndefined: false,

  // used to track if helpText has already been placed
  isHelpTextPlaced: false,

  initComponent: function () {
    var me = this;

    if (me.helpText && !me.isHelpTextPlaced) {
      me.beforeSubTpl = '<span class="nx-boxlabel">' + me.helpText + '</span>';
      me.isHelpTextPlaced = true;
    }

    me.callParent(arguments);
  },

  setValue: function (value) {
    var me = this;
    me.callParent(arguments);
    if (me.readOnly && me.hideIfUndefined) {
      if (value) {
        me.show();
      }
      else {
        me.hide();
      }
    }
  },

  setHelpText: function (text) {
    this.beforeSubTpl = '<span class="nx-boxlabel">' +text + '</span>';
    this.fireEvent('render');
  }

});
