
/*global Ext*/

/**
 * Quick search box
 *
 * @since 3.0
 */
Ext.define('AS.view.header.QuickSearch', {
  extend: 'AS.ext.SearchBox',
  alias: 'widget.as-header-quicksearch',
  requires: [
    'AS.I18n',
    'AS.ext.plugin.SearchBoxTip'
  ],

  plugins: [
    {
      ptype: 'searchboxtip',
      message: AS.I18n.get('SearchBoxTip_ExactMatch') + '<br>' + AS.I18n.get('SearchBoxTip_Wildcard')
    }
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      itemId: 'quicksearch',
      cls: 'as-quicksearch',
      iconClass: 'fa-search',
      width: 220,
      emptyText: AS.I18n.get('Header_QuickSearch_Empty'),
      ariaRole: 'search',
      ariaLabel: AS.I18n.get('Header_QuickSearch_Tooltip')
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
