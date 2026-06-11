
/*global Ext*/

/**
 * Quick search box
 *
 * @since 3.0
 */
Ext.define('NX.view.header.QuickSearch', {
  extend: 'NX.ext.SearchBox',
  alias: 'widget.nx-header-quicksearch',
  requires: [
    'NX.I18n',
    'NX.ext.plugin.SearchBoxTip'
  ],

  plugins: [
    {
      ptype: 'searchboxtip',
      message: NX.I18n.get('SearchBoxTip_ExactMatch') + '<br>' + NX.I18n.get('SearchBoxTip_Wildcard')
    }
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      itemId: 'quicksearch',
      cls: 'nx-quicksearch',
      iconClass: 'fa-search',
      width: 220,
      emptyText: NX.I18n.get('Header_QuickSearch_Empty'),
      ariaRole: 'search',
      ariaLabel: NX.I18n.get('Header_QuickSearch_Tooltip')
    });

    this.callParent();
  },

  onValueChange: function() {
    // NO-OP to Prevent Auto-Search
  },

  triggerSearch: function() {
    this.fireEvent('search', this, this.getValue());
  }
});
