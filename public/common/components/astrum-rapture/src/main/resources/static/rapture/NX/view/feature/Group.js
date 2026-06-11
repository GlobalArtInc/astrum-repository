
/*global Ext, NX*/

/**
 * Shows an icon display of features in the {@link NX.store.FeatureGroup} store.
 *
 * @since 3.0
 */
Ext.define('NX.view.feature.Group', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-feature-group',
  requires: [
    'NX.Icons'
  ],

  cls: [
    'nx-feature-group',
    'nx-inset'
  ],

  autoScroll: true,

  items: {
    xtype: 'container',
    frame: true,
    cls: 'nx-subsection',

    items: {
      xtype: 'dataview',

      store: 'FeatureGroup',
      tpl: [
        '<tpl for=".">',
        '<div class="item-wrap">',
        '<tpl if="iconCls">',
        '<i class="fa {[values.iconCls]} fa-2x fa-fw"></i>',
        '<span class="x-fa-icon-text">{text}</span>',
        '<tpl elseif="iconName">',
        '{[ NX.Icons.img(values.iconName, "x32") ]}',
        '<span>{text}</span>',
        '</tpl>',
        '</div>',
        '</tpl>'
      ],

      itemSelector: 'div.item-wrap',
      trackOver: true,
      overItemCls: 'x-item-over',
      selectedItemCls: 'x-item-selected'
    }
  }

});
