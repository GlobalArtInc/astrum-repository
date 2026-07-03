
/*global Ext, NX*/

/*global NX*/

/**
 * 'taskScope' form field factory.
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldTaskScopeFactory', {
  singleton: true,
  alias: [
    'nx.formfield.factory.taskScope'
  ],

  create: function(formfield) {
    return Ext.create('NX.coreui.view.task.TaskScopeFieldSet');
  }
});
