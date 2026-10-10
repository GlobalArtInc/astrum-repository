
/*global Ext, AS*/

/**
 * Shows an icon display of features in the {@link AS.store.FeatureGroup} store.
 *
 * @since 3.0
 */
Ext.define('AS.view.feature.Group', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-feature-group',
  requires: [
    'AS.Icons'
  ],

  cls: [
    'as-feature-group',
    'as-inset'
  ],

  autoScroll: true,

  items: {
    xtype: 'container',
    frame: true,
    cls: 'as-subsection',

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
        '{[ AS.Icons.img(values.iconName, "x32") ]}',
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
