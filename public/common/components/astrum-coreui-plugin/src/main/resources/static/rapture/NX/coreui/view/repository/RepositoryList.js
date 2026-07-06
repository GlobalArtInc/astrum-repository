
/*global Ext, NX*/

/**
 * Repository grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.RepositoryList', {
  extend: 'NX.coreui.view.repository.RepositoryListTemplate',
  alias: 'widget.nx-coreui-repository-list',
  requires: [
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-repository-list',

  /**
   * @override
   */
  initComponent: function() {
    this.store = 'Repository';

    this.hideSize = NX.State.getValue("astrum.repository.size", false) !== true;

    this.dockedItems = [
      {
        xtype: 'nx-actions',
        items: [
          {
            xtype: 'button',
            text: NX.I18n.get('Repository_RepositoryList_New_Button'),
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
