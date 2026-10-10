
/*global Ext*/

/**
 * A **{@link Ext.form.FieldSet}** that enable/disable contained items on expand/collapse.
 *
 * @since 3.0
 */
Ext.define('AS.ext.form.OptionalFieldSet', {
  extend: 'Ext.form.FieldSet',
  alias: 'widget.as-optionalfieldset',
  cls: 'as-optionalfieldset',

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

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.on('add', me.setupMonitorOnChange, me);

    me.callParent(arguments);

    // When state changes, repeat the evaluation
    me.on('collapse', me.enableContainedItems, me);
    me.on('expand', me.enableContainedItems, me);
    me.on('afterrender', me.enableContainedItems, me);
  },

  /**
   * @private
   */
  enableContainedItems: function (container, enable) {
    var me = this;

    if (!Ext.isDefined(enable)) {
      enable = !container.collapsed;
    }

    if (container.items) {
      container.items.each(function (item) {
        if (enable) {
          if (!item.disabledOnCollapse && !item.isXType('container')) {
            item.enable();
          }
          delete item.disabledOnCollapse;
          if (item.isXType('as-optionalfieldset')) {
            if (item.collapsedOnCollapse === false) {
              item.expand();
            }
            delete item.collapsedOnCollapse;
          }
        }
        else {
          if (!Ext.isDefined(item.disabledOnCollapse)) {
            item.disabledOnCollapse = item.isDisabled();
          }
          if (!item.isXType('container')) {
            item.disable();
          }
          if (item.isXType('as-optionalfieldset')) {
            if (!Ext.isDefined(item.collapsedOnCollapse)) {
              item.collapsedOnCollapse = item.collapsed;
            }
            item.collapse();
          }
        }
        if (!item.isXType('as-optionalfieldset')) {
          me.enableContainedItems(item, enable);
        }
        if (Ext.isFunction(item.validate)) {
          item.validate();
        }
      });
    }
  },

  /**
   * @private
   * Watch for change events for contained components in order to automatically expand the toggle/checkbox.
   */
  setupMonitorOnChange: function(container, component) {
    var me = this;

    if (me === container) {
      me.mon(component, 'change', function(field, value) {
        if (value && me.collapsed) {
          me.expand();
          if (me.checkboxCmp) {
            me.checkboxCmp.resetOriginalValue();
          }
        }
      });
    }
  }

});
