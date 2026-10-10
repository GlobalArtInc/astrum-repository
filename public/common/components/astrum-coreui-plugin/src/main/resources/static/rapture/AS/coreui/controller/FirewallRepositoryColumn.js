
/*global Ext, AS*/

/**
 * Manages firewall column shown in repository grid.
 *
 * @since 3.2
 */
Ext.define('AS.coreui.controller.FirewallRepositoryColumn', {
  extend: 'AS.app.Controller',
  requires: [
    'Ext.grid.column.Column',
    'Ext.XTemplate',
    'AS.Conditions',
    'AS.Permissions',
    'AS.I18n',
    'AS.Icons',
    'AS.ext.grid.column.Renderers',
    'AS.Windows'
  ],

  models: [
    'FirewallRepositoryStatus'
  ],
  stores: [
    'FirewallRepositoryStatus'
  ],
  refs: [
    { ref: 'list', selector: 'as-coreui-repository-list-template' }
  ],

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.getApplication().getIconController().addIcons({
      'license-alert': {
        file: 'license-alert.png',
        variants: ['x16']
      },
      'security-alert': {
        file: 'security-alert.png',
        variants: ['x16']
      }
    });

    me.listen({
      controller: {
        '#Permissions': {
          changed: me.refreshFirewallColumn
        }
      },
      store: {
        '#RepositoryReference': {
          load: me.loadFirewallStatus
        },
        '#FirewallRepositoryStatus': {
          load: me.refreshFirewallColumn
        }
      },
      component: {
        'as-coreui-repository-list-template': {
          afterrender: me.bindFirewallColumn
        }
      }
    });

    me.firewallColumnTemplate = Ext.create('Ext.XTemplate',
        '<div class="as-firewall-container">',
        '<tpl if="criticalCount &gt; 0">',
        '<div class="as-firewall-chiclet critical">{criticalCount}</div>',
        '</tpl>',
        '<tpl if="severeCount &gt; 0">',
        '<div class="as-firewall-chiclet severe">{severeCount}</div>',
        '</tpl>',
        '<tpl if="moderateCount &gt; 0">',
        '<div class="as-firewall-chiclet moderate">{moderateCount}</div>',
        '</tpl>',
        '<tpl if="!criticalCount && !severeCount && !moderateCount">',
        '<div style="float:left;">' + AS.I18n.get('FirewallRepositoryColumn_NoViolations') + '</div>',
        '</tpl>',
        '<tpl if="quarantinedCount &gt; 0">',
        '<div class="as-firewall-quarantine-icon"></div><div class="as-firewall-quarantine-count">{quarantinedCount}</div>',
        '</tpl>',
        '<div class="as-external-link"></div>',
        '</div>',
        {
          compiled : true
        }
    );

    me.firewallColumnMessageTemplate = Ext.create('Ext.XTemplate',
        '<div class="as-firewall-container">',
        '<div style="text-align:center; width:100%;">{message}</div>',
        '</div>',
        {
          compiled : true
        }
    );

    me.firewallColumnErrorTemplate = Ext.create('Ext.XTemplate',
        '<div class="as-firewall-container" style="display:flex; justify-content:center; align-items:center; gap:5px;">',
        '<div class="as-firewall-iq-error-icon" style="float:none; margin-left:0;"></div>',
        '<div class="as-firewall-error" style="padding-left:0;">{errorMessage}</div>',
        '</div>',
        {
          compiled : true
        }
    );
  },

  /**
   * Load Firewall status store if repository grid is active.
   *
   * @private
   */
  loadFirewallStatus: function() {
    var me = this,
        list = me.getList();

    if (list) {
      me.getFirewallRepositoryStatusStore().load();
    }
  },

  /**
   * Add Firewall column.
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList} grid repository grid
   */
  bindFirewallColumn: function(grid) {
    var me = this;
    grid.mon(
        AS.Conditions.and(
            AS.Conditions.watchState('user'),
            AS.Conditions.isPermitted("astrum:iq-violation-summary:read")
        ),
        {
          satisfied: Ext.pass(me.addFirewallColumn, grid),
          unsatisfied: Ext.pass(me.removeFirewallColumn, grid),
          scope: me
        }
    );
  },

  /**
   * Add Firewall column to repository grid.
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList} grid repository grid
   */
  addFirewallColumn: function(grid) {
    var me = this,
        view = grid.getView();

    if (!grid.firewallColumn) {
      grid.firewallColumn = grid.pushColumn({
        id: 'firewallColumn',
        header: AS.I18n.get('FirewallRepositoryColumn_Header'),
        hideable: false,
        sortable: false,
        menuDisabled: true,
        stateId: 'firewall',
        width: 120,
        align: 'center',
        renderer: Ext.bind(me.renderFirewallColumn, me),
        listeners: {
          click: Ext.bind(me.viewReportHandler, me)
        }
      });

      view.refresh();
    }
  },

  /**
   * Remove Firewall column from repository grid
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList}
   */
  removeFirewallColumn: function(grid) {
    var column = grid.firewallColumn;
    if (column) {
      grid.headerCt.remove(column);
      grid.getView().refresh();
      delete grid.firewallColumn;
    }
  },


  /**
   * Render Firewall column based on corresponding {AS.coreui.model.FirewallRepositoryStatus}.
   *
   * @private
   * @param value (not used)
   * @param metadata firewall column metadata
   * @param {AS.coreui.model.Repository} repositoryModel repository model
   * @returns {String} Firewall column content
   */
  renderFirewallColumn: function(value, metadata, repositoryModel) {
    var me = this,
        statusModel = me.getFirewallRepositoryStatusStore().getById(repositoryModel.getId());
    if (statusModel) {
      if (statusModel.get('errorMessage')) {
        var errorMessage = Ext.util.Format.htmlEncode(statusModel.get('errorMessage'));
        metadata.tdAttr = 'data-qtip="' + errorMessage + '"';
        return me.firewallColumnErrorTemplate.apply({
          errorMessage: errorMessage
        });
      }
      // ASTRUM-54323: 'message' is the non-admin badge ("Audit enabled" / "Quarantine enabled").
      // Backend guarantees message == null on the admin path so this branch is skipped and the
      // chiclet template below renders instead. See FirewallRepositoryStatusComponent#getRepositoryStatusXO.
      else if (statusModel.get('message')) {
        var message = Ext.util.Format.htmlEncode(statusModel.get('message'));
        metadata.tdAttr = 'data-qtip="' + message + '"';
        return me.firewallColumnMessageTemplate.apply({
          message: message
        });
      }
      return me.firewallColumnTemplate.apply({
        criticalCount: statusModel.get('criticalComponentCount'),
        severeCount: statusModel.get('severeComponentCount'),
        moderateCount: statusModel.get('moderateComponentCount'),
        quarantinedCount: statusModel.get('quarantinedComponentCount')
      });
    }
    else if (me.getFirewallRepositoryStatusStore().loaded) {
      return AS.ext.grid.column.Renderers.optionalData(null);
    }
    return AS.I18n.get('FirewallRepositoryColumn_Loading');
  },

  /**
   * Refresh repository grid view (when Firewall status changes).
   *
   * @private
   */
  refreshFirewallColumn: function() {
    var list = this.getList();

    if (list) {
      list.getView().refresh();
    }
  },

  /**
   * Handler for when the cell is clicked and opens a new window to go to the IQ report
   *
   * @private
   * @param gridView (not used)
   * @param cell (not used)
   * @param row (not used)
   * @param col (not used)
   * @param event (not used)
   * @param {AS.coreui.model.Repository} repositoryModel repository model
   */
  viewReportHandler: function(gridView, cell, row, col, event, repositoryModel) {
    var me = this;
    var statusModel = me.getFirewallRepositoryStatusStore().getById(repositoryModel.getId());
    if (statusModel && statusModel.get('reportUrl')) {
      AS.Windows.open(statusModel.get('reportUrl'), 'noopener,_blank');
    }
    return false;
  }

});
