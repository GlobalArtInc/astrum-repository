
/*global Ext, AS*/

/**
 * Add repository window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositoryAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-repository-add',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  recipe: undefined,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.settingsForm = {
      xtype: 'as-coreui-repository-' + me.recipe.getId(),
      api: {
        submit: 'AS.direct.coreui_Repository.create'
      },
      settingsFormSuccessMessage: function(data) {
        return AS.I18n.get('Repository_RepositoryAdd_Create_Success') + data['name'];
      },
      editableCondition: AS.Conditions.isPermitted('astrum:repository-admin:*:*:add'),
      editableMarker: AS.I18n.get('Repository_RepositoryAdd_Create_Error'),

      buttons: [
        { text: AS.I18n.get('Repository_RepositoryList_New_Button'), action: 'add', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('Add_Cancel_Button'), action: 'back' }
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
    var storage = me.down('as-coreui-repository-storage-facet').down('combo');
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
