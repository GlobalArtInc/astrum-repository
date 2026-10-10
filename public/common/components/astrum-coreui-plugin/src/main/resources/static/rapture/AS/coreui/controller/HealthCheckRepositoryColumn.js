
/*global Ext, AS*/

/**
 * Manages health check column shown in repository grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.HealthCheckRepositoryColumn', {
  extend: 'AS.app.Controller',
  requires: [
    'Ext.grid.column.Column',
    'Ext.ToolTip',
    'AS.Conditions',
    'AS.Permissions',
    'AS.I18n',
    'AS.Icons',
    'AS.ext.grid.column.Renderers',
    'AS.coreui.util.HealthCheckUtil'
  ],

  models: [
    'HealthCheckRepositoryStatus'
  ],
  stores: [
    'HealthCheckRepositoryStatus'
  ],
  views: [
    'healthcheck.HealthCheckSummary',
    'healthcheck.HealthCheckEula'
  ],
  refs: [
    { ref: 'list', selector: 'as-coreui-repository-list-template' },
    { ref: 'summary', selector: 'as-coreui-healthcheck-summary' }
  ],
  analyzeButtonTemplate: new Ext.Template('<a class="x-btn x-unselectable x-btn-as-primary-small" ' +
      'hidefocus="on" unselectable="on" role="button" aria-hidden="false" aria-disabled="false">' +
      '<span data-ref="btnInnerEl" unselectable="on" class="x-btn-inner x-btn-inner-as-primary-small">' +
      '{0}</span></a>').compile(),

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
          changed: me.refreshHealthCheckColumn
        }
      },
      store: {
        '#RepositoryReference': {
          load: me.loadHealthCheckStatus
        },
        '#HealthCheckRepositoryStatus': {
          load: me.refreshHealthCheckColumn
        }
      },
      component: {
        'as-coreui-repository-list-template': {
          afterrender: me.bindHealthCheckColumn,
          destroy: me.destroyToolTip
        }
      }
    });
  },

  /**
   * Load Health Check status store if repository grid is active.
   *
   * @private
   */
  loadHealthCheckStatus: function() {
    var me = this,
        list = me.getList();

    if (list && AS.Permissions.check("astrum:healthcheck:read")) {
      me.getHealthCheckRepositoryStatusStore().load();
    }
  },

  /**
   * Add/Remove Health Check column based on astrum:healthcheck:read permission.
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList} grid repository grid
   */
  bindHealthCheckColumn: function(grid) {
    var me = this;
    grid.mon(
        AS.Conditions.and(
            AS.Conditions.watchState('user'),
            AS.Conditions.isPermitted("astrum:healthcheck:read"),
            AS.Conditions.hasNoFirewall()
        ),
        {
          satisfied: Ext.pass(me.addHealthCheckColumn, grid),
          unsatisfied: Ext.pass(me.removeHealthCheckColumn, grid),
          scope: me
        }
    );
  },

  /**
   * Add Health Check column to repository grid.
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList} grid repository grid
   */
  addHealthCheckColumn: function(grid) {
    var me = this,
        view = grid.getView();

    if (!grid.healthCheckColumn) {
      grid.healthCheckColumn = grid.pushColumn({
        id: 'healthCheckColumn',
        header: AS.I18n.get('HealthCheckRepositoryColumn_Header'),
        hideable: false,
        sortable: false,
        menuDisabled: true,
        stateId: 'healthcheck',
        width: 170,
        align: 'center',
        renderer: Ext.bind(me.renderHealthCheckColumn, me),
        listeners: {
          click: Ext.bind(me.maybeAskToEnable, me)
        }
      });

      view.refresh();

      grid.healthCheckTooltip = Ext.create('Ext.ToolTip', {
        target: view.getEl(),
        delegate: view.getCellSelector(grid.healthCheckColumn),
        renderTo: AS.global.document.body,
        maxWidth: 550,
        // The following offset moves the tooltip to the left of the mouse
        mouseOffset: [-15, 1],
        showDelay: 400,
        hideDelay: 2000,
        dismissDelay: 20000,
        listeners: {
          beforeshow: Ext.bind(me.updateHealthCheckColumnTooltip, me)
        }
      });
    }
  },

  /**
   * Remove ToolTip associated with the Health Check column of the grid to prevent memory leak
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList} grid repository grid
   */
  destroyToolTip: function(grid) {
    var tip = grid.healthCheckTooltip;
    if (tip) {
      tip.destroy();
    }
  },

  /**
   * Remove Health Check column from repository grid.
   *
   * @private
   * @param {AS.coreui.view.repository.RepositoryList} grid repository grid
   */
  removeHealthCheckColumn: function(grid) {
    var column = grid.healthCheckColumn;
    if (column) {
      grid.headerCt.remove(column);
      grid.getView().refresh();
      delete grid.healthCheckColumn;
      grid.healthCheckTooltip.destroy();
    }
  },

  /**
   * Render Health Check column based on corresponding {AS.coreui.model.HealthCheckRepositoryStatus}.
   *
   * @private
   * @param value (not used)
   * @param metadata Health Check column metadata
   * @param {AS.coreui.model.Repository} repositoryModel repository model
   * @returns {String} Health Check column content
   */
  renderHealthCheckColumn: function(value, metadata, repositoryModel) {
    var me = this,
        statusModel = me.getHealthCheckRepositoryStatusStore().getById(repositoryModel.getId());

    if (!statusModel) {
      if (!me.getHealthCheckRepositoryStatusStore().loaded && AS.Permissions.check('astrum:healthcheck:read')) {
        return AS.I18n.get('HealthCheckRepositoryColumn_Loading');
      }
      else {
        // RHC not supported for this repository
        return AS.ext.grid.column.Renderers.optionalData(null);
      }
    }
    else if (statusModel.get('enabled')) {
      if (statusModel.get('analyzing')) {
        return AS.I18n.get('HealthCheckRepositoryColumn_Analyzing');
      }
      return '<div class="repository-health-check"><img src="' + AS.Icons.url('security-alert', 'x16') +
          '">&nbsp;<span class="security-alert">' + statusModel.get('securityIssueCount') + '</span>&nbsp;&nbsp;' +
          '<img src="' + AS.Icons.url('license-alert', 'x16') + '" style="margin-left:10px">&nbsp;' +
          '<span class="license-alert">' + statusModel.get('licenseIssueCount') + '</span></div>';
    }
    else if (AS.Permissions.check('astrum:healthcheck:update')) {
      return me.analyzeButtonTemplate.apply([AS.I18n.get('HealthCheckRepositoryColumn_Analyze')]);
    }
    else {
      // User doesn’t have the permissions to enable RHC
      return AS.ext.grid.column.Renderers.optionalData(null);
    }
  },

  /**
   * Update Health Check column tooltip based on {AS.coreui.model.HealthCheckRepositoryStatus}.
   *
   * @private
   * @param {Ext.tip.Tooltip} tip
   * @returns {boolean} true if tooltip should be shown
   */
  updateHealthCheckColumnTooltip: function(tip) {
    var me = this,
        view = me.getList().getView(),
        repository, status, html, cell;

    if (tip.triggerElement) {
      repository = view.getRecord(tip.triggerElement.parentNode);
      if (repository) {
        status = me.getHealthCheckRepositoryStatusStore().getById(repository.getId());
        if (status) {
          if (status.get('enabled')) {
            if (status.get('analyzing')) {
              html = AS.I18n.get('HealthCheckRepositoryColumn_Analyzing_Tooltip');
            }
            else {
              if (AS.Permissions.check('astrum:healthchecksummary:read')) {
                cell = view.getCell(repository, me.getList().healthCheckColumn, true);
                Ext.defer(me.showSummary, 0, me, [status, cell.getX(), cell.getY()]);
                return false;
              }
              html = AS.I18n.get('HealthCheckRepositoryColumn_View_Permission_Error');
            }
          }
          else if (AS.Permissions.check('astrum:healthcheck:update')) {
            html = AS.I18n.get('HealthCheckRepositoryColumn_Analyze_Tooltip');
          }
          else {
            html = AS.I18n.get('HealthCheckRepositoryColumn_Analyze_Permission_Error');
          }
        }
        else if (me.getHealthCheckRepositoryStatusStore().loaded) {
          html = AS.I18n.get('HealthCheckRepositoryColumn_Unavailable_Tooltip');
        }
        tip.update(html);
        return true;
      }
    }
    return false;
  },

  /**
   * Refresh repository grid view (when Health Check status changes).
   *
   * @private
   */
  refreshHealthCheckColumn: function() {
    var list = this.getList();

    if (list) {
      list.getView().refresh();
    }
  },

  /**
   * Show Health Check summary window.
   *
   * @private
   * @param {AS.coreui.model.HealthCheckRepositoryStatus} statusModel Health Check status model
   * @param x where summary window should be shown
   * @param y where summary window should be shown
   */
  showSummary: function(statusModel, x, y) {
    var me = this,
        summary = me.getSummary(),
        docks;

    if (!summary) {
      summary = Ext.widget({
        xtype: 'as-coreui-healthcheck-summary',
        x: x,
        y: y,
        height: statusModel.get('iframeHeight') + 8,
        width: statusModel.get('iframeWidth') + 8,
        statusModel: statusModel
      });
      docks = summary.getDockedItems('toolbar[dock="bottom"]');
      Ext.each(docks, function(dock) {
        summary.setHeight(summary.getHeight() + dock.getHeight());
      });
    }
  },

  /**
   * Ask user if Health Check should be enabled, if applicable and not already enabled.
   *
   * @private
   */
  maybeAskToEnable: function(gridView, cell, row, col, event, repositoryModel) {
    var me = this,
        list = me.getList(),
        status = me.getHealthCheckRepositoryStatusStore().getById(repositoryModel.getId());

    if (status && !status.get('enabled') && AS.Permissions.check('astrum:healthcheck', 'update')) {
      list.healthCheckTooltip.hide();
      Ext.Msg.show({
        title: AS.I18n.get('HealthCheckRepositoryColumn_Analyze_Dialog_Title'),
        msg: AS.I18n.format('HealthCheckRepositoryColumn_Analyze_Dialog_Msg', Ext.util.Format.htmlEncode(repositoryModel.get('name'))),
        buttons: 7, // OKYESNO
        buttonText: {
          ok: AS.I18n.get('HealthCheckRepositoryColumn_Analyze_Dialog_Ok_Text'),
          yes: AS.I18n.get('HealthCheckRepositoryColumn_Analyze_Dialog_Yes_Text')
        },
        icon: Ext.MessageBox.QUESTION,
        closeable: false,
        fn: function(buttonName) {
          if (buttonName === 'yes' || buttonName === 'ok') {
            if (status.get('eulaAccepted')) {
              me.enableAnalysis(buttonName === 'yes' ? status.getId() : undefined);
            }
            else {
              Ext.widget('as-coreui-healthcheck-eula', {
                acceptFn: function() {
                  me.enableAnalysis(buttonName === 'yes' ? status.getId() : undefined);
                }
              });
            }
          }
        }
      });
    }
    return false;
  },

  /**
   * Enable Health Check for specified repository or all if repository not specified.
   *
   * @private
   * @param {String} repositoryName to enable Health Check for, or undefined if to enable for all repositories.
   */
  enableAnalysis: function(repositoryName) {
    var me = this;

    if (repositoryName) {
      AS.direct.healthcheck_Status.update(
          true /* eula accepted */, repositoryName, true /* enabled */,
          function(response) {
            if (Ext.isObject(response) && response.success) {
              me.getHealthCheckRepositoryStatusStore().load();
            }
          }
      );
    }
    else {
      AS.direct.healthcheck_Status.enableAll(
          true /* eula accepted */,
          function(response) {
            if (Ext.isObject(response) && response.success) {
              me.getHealthCheckRepositoryStatusStore().load();
            }
          }
      );
    }
  }

});
