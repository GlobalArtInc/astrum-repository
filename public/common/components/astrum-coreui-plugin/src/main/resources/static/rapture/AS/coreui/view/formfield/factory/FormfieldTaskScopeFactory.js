
/*global Ext, AS*/

/*global AS*/

/**
 * 'taskScope' form field factory.
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldTaskScopeFactory', {
  singleton: true,
  alias: [
    'as.formfield.factory.taskScope'
  ],

  create: function(formfield) {
    return Ext.create('AS.coreui.view.task.TaskScopeFieldSet');
  }
});
