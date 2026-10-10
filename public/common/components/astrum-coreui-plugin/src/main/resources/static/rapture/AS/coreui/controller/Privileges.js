
/*global Ext, AS*/

/**
 * Privilege controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Privileges', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.view.info.Panel',
    'AS.view.info.Entry',
    'AS.Conditions',
    'AS.Messages',
    'AS.Permissions',
    'AS.I18n'
  ],
  masters: [
    'as-coreui-privilege-list'
  ],
  stores: [
    'Privilege',
    'PrivilegeType'
  ],
  models: [
    'Privilege',
    'PrivilegeType'
  ],
  views: [
    'privilege.PrivilegeAdd',
    'privilege.PrivilegeFeature',
    'privilege.PrivilegeList',
    'privilege.PrivilegeSelectType',
    'privilege.PrivilegeSettings',
    'privilege.PrivilegeSettingsForm',
    'formfield.SettingsFieldSet'
  ],
  refs: [
    {ref: 'feature', selector: 'as-coreui-privilege-feature'},
    {ref: 'content', selector: 'as-feature-content' },
    {ref: 'list', selector: 'as-coreui-privilege-list'},
    {ref: 'settings', selector: 'as-coreui-privilege-feature as-coreui-privilege-settings'}
  ],
  icons: {
    'privilege-default': {
      file: 'medal_gold_red.png',
      variants: ['x16', 'x32']
    },
    'privilege-application': {
      file: 'medal_gold_green.png',
      variants: ['x16', 'x32']
    },
    'privilege-wildcard': {
      file: 'medal_gold_blue.png',
      variants: ['x16', 'x32']
    },
    'privilege-repository': {
      file: 'database.png',
      variants: ['x16', 'x32']
    },
    'privilege-repository-admin': {
      file: 'database_red.png',
      variants: ['x16', 'x32']
    },
    'privilege-repository-content-selector': {
      file: 'content_selector.png',
      variants: ['x16', 'x32']
    },
    'privilege-repository-view': {
      file: 'database.png',
      variants: ['x16', 'x32']
    },
    'privilege-script': {
      file: 'script_text.png',
      variants: ['x16', 'x32']
    }
  },

  permission: 'astrum:privileges',

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.features = {
      mode: 'admin',
      path: '/Security/Privileges',
      text: AS.I18n.get('Privileges_Text'),
      description: AS.I18n.get('Privileges_Description'),
      view: {xtype: 'as-coreui-privilege-feature'},
      iconConfig: {
        file: 'medal_gold_green.png',
        variants: ['x16', 'x32']
      },
      visible: function() {
        return AS.Permissions.check('astrum:privileges:read') &&
            !AS.State.getValue('astrum.react.privileges', false);
      },
      weight: 10
    };

    me.callParent();

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        }
      },
      store: {
        '#Privilege': {
          load: function() {
            var focusedElement = window.document.activeElement;

            me.reselect(arguments);

            // Calling reselect() removes the focus from the Filter box.
            // This prevents the Filter box from losing focus while the
            // user is typing in it - ASTRUM-16975, ASTRUM-12693
            if (focusedElement) {
              focusedElement.focus();
            }
          }
        }
      },
      component: {
        'as-coreui-privilege-list': {
          beforerender: me.loadPrivilegeStores
        },
        'as-coreui-privilege-list button[action=new]': {
          click: me.showSelectTypePanel
        },
        'as-coreui-privilege-selecttype': {
          cellclick: me.showAddPanel
        },
        'as-coreui-privilege-add button[action=add]': {
          click: me.createPrivilege
        },
        'as-coreui-privilege-settings button[action=save]': {
          click: me.updatePrivilege
        }
      }
    });
  },

  /**
   * @override
   */
  getDescription: function (model) {
    return model.get('name');
  },

  /**
   * @override
   */
  onSelection: function (list, model) {
    var settingsPanel = this.getSettings();
    if (Ext.isDefined(model)) {
      settingsPanel.loadRecord(model);
    }
  },

  /**
   * @private
   */
  showAddPanel: function(list, td, cellIndex, model) {
    var me = this,
        panel;

    // Shows the first panel in the create wizard, and sets the breadcrumb
    me.setItemName(2, AS.I18n.format('Privileges_Create_Title', model.get('name')));
    me.loadCreateWizard(2, panel = Ext.create('widget.as-coreui-privilege-add'));
    var m = me.getPrivilegeModel().create({ type: model.getId(), readonly: false });
    panel.down('as-settingsform').loadRecord(m);
  },

  /**
   * @override
   * @protected
   * Enables 'New' button when user has 'create' permission and there is at least one privilege type.
   */
  bindNewButton: function(button) {
    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted(this.permission + ':create'),
            AS.Conditions.storeHasRecords('PrivilegeType')
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
   * @protected
   * Enables 'Delete' when user has 'delete' permission and privilege is not read only.
   */
  bindDeleteButton: function (button) {
    var me = this;

    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted(me.permission + ':delete'),
            AS.Conditions.watchEvents([
              { observable: me.getStore('Privilege'), events: ['load']},
              { observable: Ext.History, events: ['change']}
            ], me.watchEventsHandler())
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
   * @private
   */
  watchEventsHandler: function () {
    var me = this,
        store = me.getStore('Privilege');

    return function() {
      var privilegeId = me.getModelIdFromBookmark(),
          model = privilegeId ? store.findRecord('id', privilegeId, 0, false, true, true) : undefined;

      if (model) {
        return !model.get('readOnly');
      }

      return false;
    };
  },

  /**
   * @private
   * Creates a privlege.
   */
  createPrivilege: function(button) {
    var me = this,
        form = button.up('form'),
        values = form.getValues();

    AS.direct.coreui_Privilege.create(values, function(response) {
      if (Ext.isObject(response)) {
        if (response.success) {
          AS.Messages.success(AS.I18n.format('Privileges_Create_Success',
                me.getDescription(me.getPrivilegeModel().create(response.data))));
          me.getStore('Privilege').load();
        }
        else if (Ext.isDefined(response.errors)) {
          form.markInvalid(response.errors);
        }
      }
    });
  },

  /**
   * @private
   * Updates privilege.
   */
  updatePrivilege: function(button) {
    var me = this,
        form = button.up('form'),
        values = form.getValues();

    me.getContent().getEl().mask(AS.I18n.get('Privileges_Update_Mask'));
    AS.direct.coreui_Privilege.update(values, function(response) {
      me.getContent().getEl().unmask();
      if (Ext.isObject(response)) {
        if (response.success) {
          AS.Messages.success(AS.I18n.format('Privileges_Update_Success',
                me.getDescription(me.getPrivilegeModel().create(response.data))));
          form.fireEvent('submitted', form);
          me.getStore('Privilege').load();
        }
        else if (Ext.isDefined(response.errors)) {
          form.markInvalid(response.errors);
        }
      }
    });
  },

  /**
   * @private
   * @override
   * Deletes a privilege.
   * @param model privilege to be deleted
   */
  deleteModel: function (model) {
    var me = this;
    AS.direct.coreui_Privilege.remove(model.getId(), function (response) {
      me.getStore('Privilege').load();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('Privileges_Delete_Success', model.get('name')));
      }
    });
  },

  /**
   * @private
   */
  showSelectTypePanel: function() {
    var me = this;

    // Shows the first panel in the create wizard, and sets the breadcrumb
    me.setItemName(1, AS.I18n.get('Privileges_Select_Title'));
    me.loadCreateWizard(1, Ext.widget({
      xtype: 'panel',
      layout: {
        type: 'vbox',
        align: 'stretch',
        pack: 'start'
      },
      items: [
        {
          xtype: 'as-coreui-privilege-selecttype',
          flex: 1
        }
      ]
    }));
  },

  loadPrivilegeStores: function() {
    var me = this;

    Ext.each(this.storesForLoad, function(store){
      me.getStore(store).clearFilter(true);
    });
    me.loadStores();
  }
});
