/// A PixelScript Object.
/// 
/// The way this works is via the host, a Pseudo type can be created. So when the scripting
/// language interacts with the object, it calls it's pseudo methods.
/// 
/// example:
/// ```c
/// struct Person {
///     const char* name;
///     int age;
/// 
///     Person(const char* name, int age) {
///         this->name = name;
///         this->age = age;
///     }
/// 
///     void set_name(const char* name) {
///         this->name = name;
///     }
/// 
///     void set_age(int age) {
///         this->age = age;
///     }
/// 
///     int get_age() {
///         return this->age;
///     }
/// 
///     const char* get_name() {
///         return this->name;
///     }
/// };
/// 
/// void free_person(Person* p) {
///     // TODO
/// }
/// 
/// Class* person_class = pixelscript_new_class("Person");
/// pixelscript_class_add_constructor(person_class, )
/// 
/// Var* new_person(int argc, Var** argv, void* opaque) {
///     return Var{}
/// }
/// 
/// PixelObject* object_ptr = pixelscript_new_object();
/// pixelscript_set_constructor(object_ptr, Person::Person);
/// pixelscript_set_freemethod(object_ptr, free_person);
/// pixelscript_set_callback(object_ptr, "set_name", Person::set_name);
/// pixelscript_set_callback(object_ptr, "set_age", Person::set_age);
/// ``` 
pub struct PixelObject {

}