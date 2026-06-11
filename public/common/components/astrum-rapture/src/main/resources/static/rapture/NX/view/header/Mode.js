
/*global Ext*/

/**
 * Mode selector widget.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.Mode', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-header-mode',

  config: {
    /**
     * Mode name.
     *
     * @cfg {String}
     */
    name: undefined,

    /**
     * Mode menu title.
     *
     * @cfg {String}
     */
    title: undefined,

    /**
     * Mode button text.
     *
     * @cfg {String}
     */
    text: undefined,

    /**
     * Mode button tooltip.
     *
     * @cfg {String}
     */
    tooltip: undefined,

    /**
     * Mode icon class
     *
     * @cfg {String}
     */
    iconCls: undefined,


    /**
     * If button should auto hide when no features are available for selected mode.
     *
     * @cfg {boolean}
     */
    autoHide: false,

    /**
     * If menu should be collapsed automatically when mode is selected.
     *
     * @cfg {boolean}
     */
    collapseMenu: false
  },

  publishes: {
    text: true,
    tooltip: true
  },

  /**
   * Absolute layout for caret positioning over button.
   */
  // layout: 'absolute',
  // TODO: Absolute layout breaks on the latest versions of ExtJS 6 so we'll need a different way to accomplish this

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.setViewModel({
      data: {
        text: me.getText(),
        tooltip: me.getTooltip()
      }
    });

    Ext.apply(me, {
      items: [
        {
          xtype: 'button',
          ui: 'nx-mode',
          cls: 'nx-modebutton',
          scale: 'medium',
          // min-width here as the user-mode extends past this with user-name
          minWidth: 49,
          toggleGroup: 'mode',
          allowDepress: false,
          handler: function(button) {
            me.fireEvent('selected', me);
          },
          iconCls: me.iconCls,
          // copied autoEl from Ext.button.Button
          autoEl: {
            tag: 'a',
            hidefocus: 'on',
            unselectable: 'on',
            // expose mode name on element for testability to target button by mode name
            'data-name': me.name
          },

          bind: {
            text: '{text:htmlEncode}',
            tooltip: '{tooltip:htmlEncode}'
          },

          ariaLabel: Ext.String.htmlEncode(me.text ? me.text : me.title)
        }
      ]
    });

    me.callParent();
  },

  /**
   * @public
   * @param {boolean} state
   * @param {boolean} suppressEvent
   */
  toggle: function(state, suppressEvent) {
    this.down('button').toggle(state, suppressEvent);
  }
});
