
/*global Ext, AS*/

/**
 * Tasks controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Tasks', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.view.info.Panel',
    'AS.view.info.Entry',
    'AS.Conditions',
    'AS.Dialogs',
    'AS.Messages',
    'AS.Permissions',
    'AS.I18n'
  ],
  masters: [
    'as-coreui-task-list'
  ],
  stores: [
    'Task',
    'TaskType',
    'Repository',
    'Blobstore',
    'ReconcilePlans'
  ],
  models: [
    'Task'
  ],
  views: [
    'task.TaskAdd',
    'task.TaskFeature',
    'task.TaskList',
    'task.TaskSelectType',
    'task.TaskScheduleFieldSet',
    'task.TaskScheduleFields',
    'task.TaskScheduleAdvanced',
    'task.TaskScheduleDaily',
    'task.TaskScheduleHourly',
    'task.TaskScheduleManual',
    'task.TaskScheduleMonthly',
    'task.TaskScheduleOnce',
    'task.TaskScheduleWeekly',
    'task.TaskScopeFields',
    'task.TaskScopeFieldSet',
    'task.TaskScopeDates',
    'task.TaskScopeDuration',
    'task.TaskSummary',
    'task.TaskSettings',
    'task.TaskSettingsForm',
    'formfield.SettingsFieldSet'
  ],
  refs: [
    { ref: 'feature', selector: 'as-coreui-task-feature' },
    { ref: 'content', selector: 'as-feature-content' },
    { ref: 'list', selector: 'as-coreui-task-list' },
    { ref: 'summary', selector: 'as-coreui-task-summary' },
    { ref: 'settings', selector: 'as-coreui-task-settings' }
  ],
  icons: {
    'task-default': {
      file: 'time.png',
      variants: ['x16', 'x32']
    }
  },

  permission: 'astrum:tasks',

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.features = {
      mode: 'admin',
      path: '/System/Tasks',
      text: AS.I18n.get('Tasks_Text'),
      description: AS.I18n.get('Tasks_Description'),
      view: {xtype: 'as-coreui-task-feature'},
      iconConfig: {
        file: 'time.png',
        variants: ['x16', 'x32']
      },
      visible: function() {
        return AS.Permissions.check('astrum:tasks:read') &&
            !AS.State.getValue('astrum.react.tasks', false);
      },
    };

    me.callParent();

    me.listen({
      store: {
        '#Task': {
          load: me.reselect
        }
      },
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        }
      },
      component: {
        'as-coreui-task-list': {
          beforerender: me.loadStores
        },
        'as-coreui-task-list button[action=new]': {
          click: me.showSelectTypePanel
        },
        'as-coreui-task-feature button[action=run]': {
          runaction: me.runTask,
          afterrender: me.bindRunButton
        },
        'as-coreui-task-feature button[action=stop]': {
          runaction: me.stopTask,
          afterrender: me.bindStopButton
        },
        'as-coreui-task-settings button[action=save]': {
          click: me.updateTask
        },
        'as-coreui-task-add button[action=add]': {
          click: me.createTask
        },
        'as-coreui-task-selecttype': {
          cellclick: me.showAddPanel
        },
        'combobox[name=property_fromGroup]': {
          change: me.removeGroupMemberTaskFromGroupChanged
        },
        'combobox[name=property_moveRepositoryName]': {
          change: me.moveRepositoryTaskRepositoryNameChanged
        }
      }
    });
  },

  onNavigate: function() {
    var bookmark = AS.Bookmarks.getBookmark();
    var token = bookmark && bookmark.getToken ? bookmark.getToken() : null;
    if (token && token.includes('system/tasks')) {
      this.reselect();
    }
  },

  /**
   * @override
   * Returns a description of task suitable to be displayed.
   * @param {AS.coreui.model.Task} model selected model
   */
  getDescription: function(model) {
    return model.get('name') + ' (' + model.get('typeName') + ')';
  },

  /**
   * @override
   * Load task model into detail tabs.
   * @param {AS.coreui.view.task.TaskList} list task grid
   * @param {AS.coreui.model.Task} model selected model
   */
  onSelection: function(list, model) {
    var me = this,
        settings = me.getSettings(),
        taskTypeModel,
        taskTypeStore = me.getStore('TaskType');

    if (Ext.isDefined(model)) {
      me.showSummary(model);
      if (!taskTypeStore.isLoaded()) {
        taskTypeStore.on('load', function() {
          me.onSelection(list, model);
        }, me, {single: true});
        return;
      }
      taskTypeModel = taskTypeStore.getById(model.get('typeId'));
      if (taskTypeModel) {
        if (!settings) {
          me.addTab({ xtype: 'as-coreui-task-settings', title: AS.I18n.get('Tasks_Settings_Title'), weight: 20 });
        }
        this.showSettings(model);
      }
      else {
        if (settings) {
          me.removeTab(settings);
        }
      }
    }
  },

  /**
   * @private
   * Displays task summary.
   * @param {AS.coreui.model.Task} model task model
   */
  showSummary: function(model) {
    var info = {},
        me = this,
        summary = me.getSummary();

    info[AS.I18n.get('Tasks_ID_Info')] = model.getId();
    info[AS.I18n.get('Tasks_Name_Info')] = model.get('name');
    info[AS.I18n.get('Tasks_Type_Info')] = model.get('typeName');
    info[AS.I18n.get('Tasks_Status_Info')] = model.get('statusDescription');
    info[AS.I18n.get('Tasks_NextRun_Info')] = model.get('nextRun');
    info[AS.I18n.get('Tasks_LastRun_Info')] = model.get('lastRun');
    info[AS.I18n.get('Tasks_LastResult_Info')] = model.get('lastRunResult');

    summary.showInfo(info);
  },

  /**
   * @private
   * Displays task settings, attempting to load reconcile plan information if applicable.
   * @param {AS.coreui.model.Task} model task model
   */
  showSettings: function(model) {
    var me = this;

    me.subscribeToReconciliationPlans(model);
    me.getSettings().loadRecord(model);
  },

  /**
   * @private
   */
  showSelectTypePanel: function() {
    var me = this;

    //clear any filters that may previously have been applied
    me.getStore('TaskType').clearFilter();

    // Show the first panel in the create wizard, and set the breadcrumb
    me.setItemName(1, AS.I18n.get('Tasks_Select_Title'));
    me.loadCreateWizard(1, Ext.widget({
      xtype: 'panel',
      layout: {
        type: 'vbox',
        align: 'stretch'
      },
      items: [
        {
          xtype: 'panel',
          ui: 'as-drilldown-message',
          cls: 'as-drilldown-info',
          iconCls: AS.Icons.cls('drilldown-info', 'x16'),
          title: AS.I18n.format('Task_Script_Creation_Disabled'),
          hidden: AS.State.getValue('allowScriptCreation'),
        },
        {
          xtype: 'as-coreui-task-selecttype',
          flex: 1
        }
      ]
    }));
  },

  /**
   * @private
   */
  showAddPanel: function(list, td, cellIndex, model) {
    var me = this,
      panel;

    // Show the second panel in the create wizard, and set the breadcrumb
    me.setItemName(2, AS.I18n.format('Tasks_Create_Title', model.get('name')));
    me.loadCreateWizard(2, panel = Ext.widget({
      xtype: 'panel',
      layout: {
        type: 'vbox',
        align: 'stretch',
        pack: 'start'
      },
      items: [
        {
          xtype: 'as-coreui-task-add',
          flex: 1
        }
      ]
    }));
    const taskModel = me.getTaskModel().create({typeId: model.getId(), enabled: true});

    me.subscribeToReconciliationPlans(taskModel)
    panel.down('as-settingsform').loadRecord(taskModel);
  },

  /**
   * @private
   */
  updateTask: function(button) {
    var me = this,
      form = button.up('form'),
      values = form.getValues();

    me.getContent().getEl().mask(AS.I18n.get('Tasks_Update_Mask'));
    AS.direct.coreui_Task.update(values, function(response) {
      me.getContent().getEl().unmask();
      if (Ext.isObject(response)) {
        if (response.success) {
          AS.Messages.success(AS.I18n.format('Tasks_Update_Success',
              me.getDescription(me.getTaskModel().create(response.data))));
          form.fireEvent('submitted', form);
          me.getStore('Task').load();
        }
        else if (Ext.isDefined(response.errors)) {
          form.markInvalid(response.errors);
        }
        else if (Ext.isDefined(response.message)) {
          AS.Messages.error(response.message);
        }
      }
    });
  },

  /**
   * @private
   */
  createTask: function(button) {
    var me = this,
      form = button.up('form'),
      values = form.getValues();

    AS.direct.coreui_Task.create(values, function(response) {
      if (Ext.isObject(response)) {
        if (response.success) {
          AS.Messages.success(AS.I18n.format('Tasks_Create_Success',
              me.getDescription(me.getTaskModel().create(response.data))));
          me.getStore('Task').load();
        }
        else if (Ext.isDefined(response.errors)) {
          form.markInvalid(response.errors);
        }
        else if (Ext.isDefined(response.message)) {
          AS.Messages.error(response.message);
        }
      }
    });
  },

  /**
   * @override
   * @protected
   * Enable 'New' when user has 'create' permission and there is at least one task type.
   */
  bindNewButton: function(button) {
    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted('astrum:tasks:create'),
            AS.Conditions.storeHasRecords('TaskType')
        ),
        {
          satisfied: function () {
            button.enable();
          },
          unsatisfied: function () {
            button.disable();
          }
        }
    );
  },

  /**
   * @override
   * @private
   * Enable 'Run' when user has 'read' permission and task is 'runnable'.
   */
  bindRunButton: function(button) {
    var me = this;
    button.mon(
      AS.Conditions.and(
        AS.Conditions.isPermitted('astrum:tasks:start'),
        AS.Conditions.watchEvents(me.getObservables(), me.watchEventsHandler({ run: true }))
      ),
      {
        satisfied: function () {
          button.enable();
          button.focus();
        },
        unsatisfied: function () {
          button.disable();
        },
      }
    );
  },

  /**
   * @override
   * @private
   * Enable 'Stop' when user has 'delete' permission and task is 'stoppable'.
   */
  bindStopButton: function(button) {
    var me = this;
    button.mon(
      AS.Conditions.and(
        AS.Conditions.isPermitted('astrum:tasks:stop'),
        AS.Conditions.watchEvents(me.getObservables(), me.watchEventsHandler({ stop: true }))
      ),
      {
        satisfied: function () {
          button.enable();
          button.focus();
        },
        unsatisfied: function () {
          button.disable();
        },
      }
    );
  },

  /**
   * Observables, used to read data from database.
   * @private
   */
  getObservables: function () {
    var me = this;
    return [
      {observable: me.getStore('Task'), events: ['load']},
      {observable: Ext.History, events: ['change']},
      {observable: me.getStore('ReconcilePlans'), events: ['load']}
    ];
  },

  /**
   * @private
   */
  watchEventsHandler: function (options) {
    var me = this,
        store = me.getStore('Task');

    return function() {
      var taskId = me.getModelIdFromBookmark(),
          model = taskId ? store.findRecord('id', taskId, 0, false, true, true) : undefined;

      if (model) {
        if (options.run) {
          return model.get('runnable');
        }
        else if (options.stop) {
          return model.get('stoppable');
        }
      }

      return false;
    };
  },

  /**
   * @override
   * Delete task.
   * @param model task to be deleted
   */
  deleteModel: function(model) {
    var me = this,
        description = me.getDescription(model);

    AS.direct.coreui_Task.remove(model.getId(), function(response) {
      me.getStore('Task').load();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('Tasks_Delete_Success', description));
      }
    });
  },

  /**
   * @override
   * Run selected task.
   */
  runTask: function() {
    var me = this,
        bookmark = AS.Bookmarks.getBookmark(),
        model, modelId, description;

    // Reload the form
    me.getStore('Task').load();

    modelId = decodeURIComponent(bookmark.getSegment(1));
    model = me.getList().getStore().getById(modelId);
    description = me.getDescription(model);

    if (model) {
      if (model.data.enabled) {
        description = me.getDescription(model);
        AS.Dialogs.askConfirmation(AS.I18n.get('Tasks_RunConfirm_Title'),
            AS.I18n.format('Tasks_RunConfirm_HelpText', description), function() {
              me.getContent().getEl().mask(AS.I18n.get('Tasks_Run_Mask'));
              AS.direct.coreui_Task.run(model.getId(), function(response) {
                me.getContent().getEl().unmask();
                if (Ext.isObject(response) && response.success) {
                  me.getStore('Task').load();
                  me.getStore('ReconcilePlans').load();
                  AS.Messages.success(AS.I18n.format('Tasks_Run_Success', description));
                }
              });
            }, {scope: me});
      }
      else {
        AS.Messages.warning(AS.I18n.get('Tasks_Run_Disabled'));
      }
    }
  },

  /**
   * @override
   * Stop selected task.
   */
  stopTask: function() {
    var me = this,
      bookmark = AS.Bookmarks.getBookmark(),
      model, modelId, description;

    // Reload the form
    me.getStore('Task').load();

    modelId = decodeURIComponent(bookmark.getSegment(1));
    model = me.getList().getStore().getById(modelId);
    description = me.getDescription(model);

    if (model) {
      description = me.getDescription(model);
      AS.Dialogs.askConfirmation(AS.I18n.get('Tasks_StopConfirm_Title'),
        AS.I18n.format('Tasks_StopConfirm_HelpText', description), function() {
        me.getContent().getEl().mask(AS.I18n.get('Tasks_Stop_Mask'));
        AS.direct.coreui_Task.stop(model.getId(), function(response) {
          me.getContent().getEl().unmask();
          if (Ext.isObject(response) && response.success) {
            me.getStore('Task').load();
            AS.Messages.success(AS.I18n.format('Tasks_Stop_Success', description));
          }
        });
      }, { scope: me });
    }
  },

  removeGroupMemberTaskFromGroupChanged: function(groupComboBox, newVal, old) {
    var members = groupComboBox.up().query('[name=property_memberToRemove]')[0];
    var selectedGroup = groupComboBox.getStore().getById(newVal);
    var data = Ext.Array.map(selectedGroup.data.attributes.group.members, function(m) {return {name: m, id: m};});
    members.setValue(null);
    members.getStore().setData(data);
    if(!old) {
      members.reset();
    }
  },

  moveRepositoryTaskRepositoryNameChanged: function(moveRepoComboBox, newVal, old) {
    this.getStore('Repository').load({
      scope: this,
      callback: function() {
        this.getStore('Blobstore').load({
          scope: this,
          callback: function() {
            var me = this,
                repoStore = me.getStore('Repository'),
                selectedRepo = repoStore.findRecord('name', newVal);

            if (selectedRepo) {
              var blobstoreStore = me.getStore('Blobstore'),
                  oldSelection,
                  validSelection = false,
                  blobstoresCombo = moveRepoComboBox.up().query('[name=property_moveTargetBlobstore]')[0],
                  currentBlobStore = selectedRepo.data.attributes.storage.blobStoreName,
                  validBlobstores = blobstoreStore.getRange().filter(function(item) {
                    return item.data.name !== currentBlobStore;
                  }).map(function(item) {
                    return {name: item.data.name, id: item.data.name};
                  });

              // Check if selected value was valid, if not clean
              oldSelection = blobstoresCombo.getValue();
              for (var i = 0; i < validBlobstores.length; i++) {
                if (validBlobstores[i].id === oldSelection) {
                  oldSelection = blobstoresCombo.getValue();
                  validSelection = true;
                  break;
                }
              }

              blobstoresCombo.getStore().setData(validBlobstores);
              if (!old) {
                blobstoresCombo.reset();
              }
              if (validSelection) {
                blobstoresCombo.setValue(oldSelection);
              } else {
                blobstoresCombo.setValue(null);
              }
            }
          }
        });
      }
    });
  },

  subscribeToReconciliationPlans: function(model) {
    const me = this,
        store = me.getStore('ReconcilePlans');

    var typeId = model.get('typeId');
    if ("blobstore.executeReconciliationPlan" === typeId || "blobstore.planReconciliation" === typeId) {
      store.on('load', me.updateReconciliationPlanInformation, me, { single: true });
      store.load();
    }
  },

  updateReconciliationPlanInformation: function(store, records) {
    const summary = records[0] ? records[0].data : {};
    const me = this;

    me.getContent().down("#reconcilePlanInformation").setValues({
      planCount: summary.planCount || 0,
      blobStoreCount: summary.blobStoreCount || 0,
      repositoryCount: summary.repositoryCount || 0,
      startDate: summary.minStartDate || null,
      endDate: summary.maxEndDate || null
    });
  }
});
