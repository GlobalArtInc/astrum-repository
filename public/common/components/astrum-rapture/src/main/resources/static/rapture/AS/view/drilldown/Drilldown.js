
/*global Ext, AS*/

/**
 * The foundation class for new drilldowns. Extend this.
 *
 * @since 3.0
 */
Ext.define('AS.view.drilldown.Drilldown', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-drilldown',
  itemId: 'as-drilldown',

  requires: [
    'AS.Icons'
  ],

  // List of masters to use (xtype objects)
  masters: null,

  // List of actions to use in the detail view
  asActions: null,

  items: [],

  scrollable: 'vertical',

  layout: {
    type: 'vbox',
    align: 'stretch',
    pack: 'start'
  },

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.on('beforerender', me.loadDrilldown);

    me.callParent();
  },

  /**
   * @private
   * Initialize the items array of this component
   */
  loadDrilldown: function(me) {
    var items = [],
      views;

    // Normalize the list of masters. Clone the list to avoid memory leaks.
    if (!me.masters) {
      views = [];
    }
    else if (!Ext.isArray(me.masters)) {
      views = [Ext.clone(me.masters)];
    }
    else {
      views = Ext.Array.clone(me.masters);
    }

    if (!me.skipDetail) {
      if (me.detail) {
        // Use a custom detail panel
        views.push(me.detail);
      }
      else {
        // Use the default tab panel
        views.push(
            {
              xtype: 'as-drilldown-details',
              header: false,
              plain: true,

              layout: {
                type: 'vbox',
                align: 'stretch',
                pack: 'start'
              },

              tabs: Ext.clone(me.tabs),
              asActions: Ext.isArray(me.asActions) ? Ext.Array.clone(me.asActions) : me.asActions
            }
        );
      }
    }

    // Stack all panels onto the items array
    for (var i = 0; i < views.length; ++i) {
      items.push(me.createDrilldownItem(i, views[i], undefined));
    }

    // Add components to the container
    me.add({
      xtype: 'container',

      itemId: 'drilldown-container',

      height: '100%',

      layout: {
        type: 'card',
        animate: true
      },

      items: items
    });
  },

  /**
   * @private
   * Create a new drilldown item
   */
  createDrilldownItem: function(index, browsePanel, createPanel) {
    return {
      xtype: 'as-drilldown-item',
      itemClass: this.iconCls || AS.Icons.cls(this.iconName) + (index === 0 ? '-x32' : '-x16'),
      items: [
        {
          xtype: 'container',
          layout: 'fit',
          itemId: 'browse' + index,
          items: browsePanel
        },
        {
          xtype: 'container',
          layout: 'fit',
          itemId: 'create' + index,
          items: createPanel
        },
        {
          type: 'container',
          layout: 'fit',
          itemId: 'nothin' + index
        }
      ]
    };
  }
});
