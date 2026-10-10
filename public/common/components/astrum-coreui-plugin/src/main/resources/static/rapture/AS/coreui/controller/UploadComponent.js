
/*global Ext, AS*/

/**
 * Upload controller.
 *
 * @since 3.7
 */
Ext.define('AS.coreui.controller.UploadComponent', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.Bookmarks',
    'AS.Conditions',
    'AS.Permissions',
    'AS.I18n',
    'AS.controller.ExtDirect'
  ],
  masters: [
    'as-coreui-uploadcomponentfeature as-coreui-upload-repository-list',
    'as-coreui-uploadcomponentfeature as-coreui-upload-component'
  ],
  stores: [
    'RepositoryReference',
    'UploadComponentDefinition'
  ],
  models: [
    'UploadComponentDefinition',
    'RepositoryReference'
  ],

  views: [
    'upload.UploadComponentFeature',
    'upload.UploadComponent',
    'upload.UploadRepositoryList'
  ],

  refs: [
    {ref: 'feature', selector: 'as-coreui-uploadcomponentfeature'},
    {ref: 'repositoryList', selector: 'as-coreui-uploadcomponentfeature as-coreui-upload-repository-list'},
    {ref: 'uploadComponent', selector: 'as-coreui-uploadcomponentfeature as-coreui-upload-component'},
    {ref: 'successMessage', selector: '#as-coreui-upload-success-message'}
  ],

  icons: {
    'upload': {
      file: 'upload.png',
      variants: ['x16', 'x32']
    },
    'tick': {
      file: 'tick.png',
      variants: ['x16', 'x32']
    }
  },

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.features = {
      mode: 'browse',
      path: '/Upload',
      text: AS.I18n.get('FeatureGroups_Upload_Text'),
      description: AS.I18n.get('FeatureGroups_Upload_Description'),
      view: 'AS.coreui.view.upload.UploadComponentFeature',
      group: false,
      iconCls: 'x-fa fa-upload',
      authenticationRequired: false,
      visible: function() {
          return AS.Permissions.check('astrum:component:create') &&
            !AS.State.getValue('astrum.react.upload', false);
      }
    };

    me.callParent();

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        }
      },
      component: {
        'as-coreui-uploadcomponentfeature as-coreui-upload-repository-list': {
          beforerender: me.onBeforeRender
        },
        'as-coreui-upload-component button[action=remove_upload_asset]': {
          click: me.removeUploadAsset
        },
        'as-coreui-upload-component button[action=upload]': {
          click: me.doUpload
        },
        'as-coreui-upload-component button[action=cancel]': {
          click: me.discardUpload
        },
        'as-coreui-upload-component button[action=add_asset]': {
          click: me.addAsset
        },
        'as-coreui-upload-component textfield[name$=extension]': {
          change: me.onExtensionChange
        },
        'as-coreui-upload-component checkbox[name=generate-pom]' : {
          change: me.onGeneratePomChange
        }
      }
    });
  },

  /**
   * @override
   */
  getDescription: function(model) {
    return model.get('name');
  },

  /**
   * @override
   * When a list managed by this controller is clicked, route the event to the proper handler
   */
  onSelection: function(list, model) {
    this.loadUploadPage(model);
  },

  loadUploadPage: function(repoModel) {
    var uploadComponentDefinition = this.getStore('UploadComponentDefinition').getById(repoModel.get('format'));
    this.getUploadComponent().loadRecord(uploadComponentDefinition, repoModel);
  },

  /**
   * @override
   */
  onNavigate: function() {
    if (this.getFeature()) {
      this.onBeforeRender();
    }
  },

  loadView: function (index, model) {
    this.callParent(arguments);
    if (model) {
        //redraw the panel after visible, to get around issue where file field can be drawn at invalid size
        this.loadUploadPage(model);
    }
  },

  /**
   * @private
   * Load stores based on the bookmarked URL
   */
  onBeforeRender: function() {
    var me = this,
        uploadComponent = me.getUploadComponent(),
        bookmark = AS.Bookmarks.getBookmark(),
        list_ids = bookmark.getSegments().slice(1),
        repoStore = me.getRepositoryList().getStore(),
        repoModel;

    // If the list hasn’t loaded, don't do anything
    if (!uploadComponent) {
      return;
    }

    repoStore.removeAll();

    this.getStore('UploadComponentDefinition').load(function (data) {
        var formats = [];

        data.forEach(function(def) {
          formats.push(def.get('format'));
        });

        repoStore.addFilter([{
          property: 'format',
          filterFn: function(item) {
            return formats.indexOf(item.get('format')) !== -1;
          }
        }, {
          property: 'type',
          value: 'hosted'
        }, {
          property: 'versionPolicy',
          filterFn: function(item) {
            return item.get('versionPolicy') == null || item.get('versionPolicy') !== 'SNAPSHOT';
          }
        }, {
          property: 'status',
          filterFn: function(item) {
            return item.get('status') == null || item.get('status').online !== false;
          }
        }]);
        repoStore.load(function () {
            // Load the asset upload page
            if (list_ids[1]) {
                repoModel = repoStore.getById(decodeURIComponent(list_ids[0]));
                uploadComponent.getStore().load(function () {
                    me.onModelChanged(0, repoModel);
                    me.onSelection(undefined, repoModel);
                });
            }
            // Load the asset list view or repository list view
            else {
                me.reselect();
            }
        });
    });
  },

  removeUploadAsset: function(fileUploadField) {
    var me = this;

    fileUploadField.up('#as-coreui-upload-component-assets').remove(fileUploadField.up());
    me.refreshRemoveButtonState();
    me.updatePomFileState();
  },

  doUpload: function(button) {
    var me = this,
        fp = button.up('form');

    if(fp.getForm().isValid()) {
      me.setSuccessMessage();

      fp.getForm().submit({
        waitMsg: AS.I18n.get('FeatureGroups_Upload_Wait_Message'),
        success: function(form, action) {
          var message = AS.I18n.format('FeatureGroups_Upload_Successful_Text', form.getValues().repositoryName);
          if (AS.Permissions.check('astrum:search:read')) {
            message += ", " + AS.util.Url.asLink(
                '#browse/search=' + encodeURIComponent('keyword="' + action.result.data + '"'),
                AS.I18n.get('FeatureGroups_Upload_Successful_Link_Text'), '_self');
          }
          me.setSuccessMessage(message);
          me.resetForm();
        },
        failure: function(form, action) {
          var transaction;

          if (!action.result || !action.result.length) {
            AS.Messages.error(AS.I18n.get('UploadComponent_Upload_Error'));
            console.error(action);
          }
          else {
            transaction = {
              result: action.result[0]
            };
            transaction.result.success = false;
            AS.getApplication().getExtDirectController().checkResponse(null, transaction);
          }
        }
      });
    }
  },

  setSuccessMessage: function (message) {
      var me = this,
          successMessage = me.getSuccessMessage();

      if (message) {
          successMessage.setTitle(message);
          successMessage.show();
      }
      else {
          successMessage.hide();
      }
  },

  discardUpload: function() {
    var me = this;
    me.resetForm();
    me.loadView(me.BROWSE_INDEX);
  },

  resetForm: function() {
    var me = this,
        form = me.getUploadComponent().down('form');

    form.getForm().reset();

    // remove rows
    form.query('fileuploadfield').forEach(function(fileUploadField) {
      me.removeUploadAsset(fileUploadField);
    });

    // create new row
    me.addAsset();

    // clearOnSubmit prevents normal form reset from working...
    form.down('fileuploadfield').inputEl.dom.value = '';
  },

  addAsset: function() {
    var me = this,
        uploadComponent = me.getUploadComponent(),
        form = uploadComponent.down('form');

    uploadComponent.addAssetRow();
    me.refreshRemoveButtonState();
    me.updatePomFileState();
    form.isValid();
  },

  onExtensionChange: function() {
    var me = this,
        form = me.getUploadComponent().down('form');

    me.updatePomFileState();
    form.isValid();
  },

  updatePomFileState: function() {
    var me = this,
        form = me.getUploadComponent().down('form'),
        componentCoordinatesFieldset = form.down('fieldset[title="Component coordinates"]'),
        isPomFilePresent = form.query('textfield[name$=extension][value=pom]').length !== 0;

    if (componentCoordinatesFieldset === null) {
        return;
    }

    componentCoordinatesFieldset.setDisabled(isPomFilePresent);
    if (isPomFilePresent) {
      componentCoordinatesFieldset.mask(AS.I18n.get('FeatureGroups_Upload_Form_DetailsFromPom_Mask'), 'as-mask-without-spinner');
    }
    else {
      componentCoordinatesFieldset.unmask();
    }
  },

  /**
   * @private
   * Hide remove buttons if there is only one asset displayed
   */
  refreshRemoveButtonState: function() {
    var me = this,
        buttons = me.getUploadComponent().query('button[action=remove_upload_asset]'),
        hidden = (buttons.length === 1);

    buttons.forEach(function(button) {
      button.setVisible(!hidden);
    });
  },

  /**
   * @private
   * Change disabled state of packaging field based on generate pom checkbox
   */
  onGeneratePomChange: function(element) {
    element.up('form').down('textfield[name=packaging]').setDisabled(!element.getValue());
  }
});
