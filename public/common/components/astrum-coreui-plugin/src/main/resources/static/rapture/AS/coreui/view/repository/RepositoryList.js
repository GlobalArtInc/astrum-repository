
/*global Ext, AS*/

/**
 * Repository grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositoryList', {
  extend: 'AS.coreui.view.repository.RepositoryListTemplate',
  alias: 'widget.as-coreui-repository-list',
  requires: [
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-repository-list',

  /**
   * @override
   */
  initComponent: function() {
    this.store = 'Repository';

    this.hideSize = AS.State.getValue("astrum.repository.size", false) !== true;

    this.dockedItems = [
      {
        xtype: 'as-actions',
        items: [
          {
            xtype: 'button',
            text: AS.I18n.get('Repository_RepositoryList_New_Button'),
            iconCls: 'x-fa fa-plus-circle',
            action: 'new',
            disabled: true
          }
        ]
      }
    ];

    this.callParent();
  }

});
