
/*global Ext, NX*/

/**
 * Add repository window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.RepositoryAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-repository-add',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  recipe: undefined,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-coreui-repository-' + me.recipe.getId(),
      api: {
        submit: 'NX.direct.coreui_Repository.create'
      },
      settingsFormSuccessMessage: function(data) {
        return NX.I18n.get('Repository_RepositoryAdd_Create_Success') + data['name'];
      },
      editableCondition: NX.Conditions.isPermitted('astrum:repository-admin:*:*:add'),
      editableMarker: NX.I18n.get('Repository_RepositoryAdd_Create_Error'),

      buttons: [
        { text: NX.I18n.get('Repository_RepositoryList_New_Button'), action: 'add', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    var nameField = me.down('#name');
    nameField.setReadOnly(false);

    // Add Docker-specific validation for lowercase names
    var recipeId = me.recipe.getId();
    if (recipeId && recipeId.toLowerCase().indexOf('docker') === 0) {
      nameField.validator = function(value) {
        if (value && value !== value.toLowerCase()) {
          return 'Docker repository names must be lowercase to support path-based routing';
        }
        return true;
      };
    }

    me.down('#format').setVisible(false);
    me.down('#type').setVisible(false);
    me.down('#url').setVisible(false);
    me.down('form').add({
      xtype: 'hiddenfield',
      name: 'recipe',
      value: recipeId
    });

    // do not allow repositories to select blob stores that are assigned to a blob store group
    var storage = me.down('nx-coreui-repository-storage-facet').down('combo');
    if (storage && storage.name === 'attributes.storage.blobStoreName') {
      storage.getStore().clearFilter(true);
      storage.getStore().filter([
        {
          filterFn: function(item) {
            var blobStore = item.data;
            return !blobStore.groupName;
          }
        }
      ]);
    }
  }
});
