
/*global Ext, NX*/

/**
 * Renders icon in a column.
 *
 * Icon must be registered first with NX.controller.Icon.
 *
 * @since 3.0
 */
Ext.define('NX.ext.grid.column.Icon', {
  extend: 'Ext.grid.column.Column',
  alias: 'widget.nx-iconcolumn',
  requires: [
    'Ext.DomHelper',
    'NX.Icons'
  ],

  hideable: false,
  sortable: false,
  menuDisabled: true,
  resizable: false,
  draggable: false,
  focusable: false,
  ariaRole: 'presentation',

  /**
   * @cfg {String} iconVariant
   */
  /**
   * @cfg {Number} iconHeight
   */
  /**
   * @cfg {Number} iconWidth
   */
  /**
   * @cfg {String} iconNamePrefix
   */

  /**
   * @override
   */
  defaultRenderer: function(value, meta, record) {
    var me = this,
        cls,
        height = me.iconHeight,
        width = me.iconWidth,
        spec;

    cls = me.iconCls(value, meta, record);

    if (me.iconVariant) {
      switch (me.iconVariant) {
        case 'x16':
          height = width = 16;
          break;
        case 'x32':
          height = width = 32;
          break;
      }
    }

    spec = {
      tag: 'img',
      // NOTE: Chrome is displaying borders around <img> w/o src
      src: Ext.BLANK_IMAGE_URL,
      cls: cls,
      alt: me.iconName(value, meta, record)
    };
    if (height) {
      spec.height = height;
    }
    if (width) {
      spec.width = width;
    }

    return Ext.DomHelper.markup(spec);
  },

  /**
   * @protected
   */
  iconName: function(value, meta, record) {
    return value;
  },

  /**
   * @protected
   */
  iconCls: function(value, meta, record) {
    var me = this,
        name = me.iconName(value, meta, record);

    if (me.iconNamePrefix) {
      name = me.iconNamePrefix + name;
    }

    return NX.Icons.cls(name, me.iconVariant);
  }
});
