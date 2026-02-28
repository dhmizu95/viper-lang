/**
 * Viper Object Runtime Implementation
 * Provides object creation, method dispatch, and inheritance support
 */

#include "viper_object.h"
#include <stdio.h>
#include <string.h>
#include <stdarg.h>

/* ============================================ */
/* Global Class Registry                        */
/* ============================================ */

/* Simple class registry for looking up classes by name */
#define MAX_REGISTERED_CLASSES 256

static ViperClass* registered_classes[MAX_REGISTERED_CLASSES];
static int registered_class_count = 0;

/**
 * Register a class in the global registry
 */
void vp_class_register(ViperClass* cls) {
    if (!cls || registered_class_count >= MAX_REGISTERED_CLASSES) {
        return;
    }
    registered_classes[registered_class_count++] = cls;
}

/**
 * Look up a class by name in the global registry
 */
ViperClass* vp_class_find(const char* name) {
    if (!name) return NULL;
    
    for (int i = 0; i < registered_class_count; i++) {
        if (registered_classes[i] && strcmp(registered_classes[i]->name, name) == 0) {
            return registered_classes[i];
        }
    }
    return NULL;
}

/* ============================================ */
/* Instance Creation                            */
/* ============================================ */

/**
 * Create a new instance with field initialization
 */
ViperInstance* vp_object_create(ViperClass* cls) {
    return vp_object_new(cls);
}

/**
 * Destroy an instance and free memory
 */
void vp_object_destroy(ViperInstance* obj) {
    vp_object_del(obj);
}

/* ============================================ */
/* Method Dispatch Implementation               */
/* ============================================ */

/**
 * Get a method from an instance by name
 */
ViperMethod* vp_instance_get_method(ViperInstance* obj, const char* name) {
    if (!obj || !obj->class || !name) {
        return NULL;
    }
    return vp_method_lookup(obj->class, name);
}

/**
 * Call a method on an instance with variadic arguments
 */
int64_t vp_instance_call_method(ViperInstance* obj, const char* name, int arg_count, ...) {
    if (!obj || !name) {
        return 0;
    }
    
    ViperMethod* method = vp_instance_get_method(obj, name);
    if (!method || !method->function_ptr) {
        return 0;
    }
    
    /* Build args array */
    int64_t args[16];  /* Max 16 arguments */
    if (arg_count > 16) {
        arg_count = 16;
    }
    
    va_list ap;
    va_start(ap, arg_count);
    for (int i = 0; i < arg_count; i++) {
        args[i] = va_arg(ap, int64_t);
    }
    va_end(ap);
    
    return vp_method_call(obj, method, args);
}

/* ============================================ */
/* Attribute Access                             */
/* ============================================ */

/**
 * Get an attribute value from an instance
 */
int64_t vp_instance_getattr(ViperInstance* obj, const char* name, size_t offset) {
    if (!obj || !name) {
        return 0;
    }
    
    /* First check if this is a property */
    ViperMethod* getter = vp_method_lookup(obj->class, name);
    if (getter && (getter->flags & VIPER_METHOD_PROPERTY)) {
        return vp_property_get(obj, (ViperProperty*)getter);
    }
    
    /* Otherwise, get field value */
    return VIPER_GET_FIELD(obj, int64_t, offset);
}

/**
 * Set an attribute value on an instance
 */
void vp_instance_setattr(ViperInstance* obj, const char* name, size_t offset, int64_t value) {
    if (!obj || !name) {
        return;
    }
    
    /* First check if this is a property with setter */
    ViperMethod* setter = vp_method_lookup(obj->class, name);
    if (setter && (setter->flags & VIPER_METHOD_PROPERTY_SETTER)) {
        vp_property_set(obj, (ViperProperty*)setter, value);
        return;
    }
    
    /* Otherwise, set field value */
    VIPER_SET_FIELD(obj, int64_t, offset, value);
}

/* ============================================ */
/* String and Representation                    */
/* ============================================ */

/**
 * Get string representation of an object
 */
const char* vp_object_to_string(ViperInstance* obj) {
    return vp_object_str(obj);
}

/**
 * Get repr representation of an object
 */
const char* vp_object_to_repr(ViperInstance* obj) {
    if (!obj) {
        return "<None>";
    }
    
    /* Look for __repr__ method */
    ViperMethod* repr_method = vp_method_lookup(obj->class, "__repr__");
    if (repr_method && repr_method->function_ptr) {
        int64_t args[1] = {0};
        int64_t result = vp_method_call(obj, repr_method, args);
        return (const char*)result;
    }
    
    /* Fall back to __str__ */
    return vp_object_str(obj);
}

/* ============================================ */
/* Comparison Operations                        */
/* ============================================ */

/**
 * Compare two objects for equality
 */
bool vp_object_equals(ViperInstance* a, ViperInstance* b) {
    return vp_object_eq(a, b);
}

/**
 * Compare two objects for less than
 */
bool vp_object_lt(ViperInstance* a, ViperInstance* b) {
    if (!a || !b) return false;
    
    /* Look for __lt__ method */
    ViperMethod* lt_method = vp_method_lookup(a->class, "__lt__");
    if (lt_method && lt_method->function_ptr) {
        int64_t args[1] = {(int64_t)b};
        return vp_method_call(a, lt_method, args) != 0;
    }
    
    /* Default: compare pointers */
    return a < b;
}

/**
 * Compare two objects for less than or equal
 */
bool vp_object_le(ViperInstance* a, ViperInstance* b) {
    if (!a || !b) return false;
    
    /* Look for __le__ method */
    ViperMethod* le_method = vp_method_lookup(a->class, "__le__");
    if (le_method && le_method->function_ptr) {
        int64_t args[1] = {(int64_t)b};
        return vp_method_call(a, le_method, args) != 0;
    }
    
    /* Default: compare pointers */
    return a <= b;
}

/* ============================================ */
/* Arithmetic Operations                        */
/* ============================================ */

/**
 * Add two objects
 */
ViperInstance* vp_object_add(ViperInstance* a, ViperInstance* b) {
    if (!a) return b;
    if (!b) return a;
    
    /* Look for __add__ method */
    ViperMethod* add_method = vp_method_lookup(a->class, "__add__");
    if (add_method && add_method->function_ptr) {
        int64_t args[1] = {(int64_t)b};
        int64_t result = vp_method_call(a, add_method, args);
        return (ViperInstance*)result;
    }
    
    return NULL;
}

/**
 * Subtract two objects
 */
ViperInstance* vp_object_sub(ViperInstance* a, ViperInstance* b) {
    if (!a) return b;
    if (!b) return a;
    
    /* Look for __sub__ method */
    ViperMethod* sub_method = vp_method_lookup(a->class, "__sub__");
    if (sub_method && sub_method->function_ptr) {
        int64_t args[1] = {(int64_t)b};
        int64_t result = vp_method_call(a, sub_method, args);
        return (ViperInstance*)result;
    }
    
    return NULL;
}

/**
 * Multiply two objects
 */
ViperInstance* vp_object_mul(ViperInstance* a, ViperInstance* b) {
    if (!a) return b;
    if (!b) return a;
    
    /* Look for __mul__ method */
    ViperMethod* mul_method = vp_method_lookup(a->class, "__mul__");
    if (mul_method && mul_method->function_ptr) {
        int64_t args[1] = {(int64_t)b};
        int64_t result = vp_method_call(a, mul_method, args);
        return (ViperInstance*)result;
    }
    
    return NULL;
}

/* ============================================ */
/* Container Operations                         */
/* ============================================ */

/**
 * Get length of a container object
 */
int64_t vp_object_length(ViperInstance* obj) {
    return vp_object_len(obj);
}

/**
 * Get item from container
 */
int64_t vp_object_getitem(ViperInstance* obj, int64_t index) {
    if (!obj) return 0;
    
    /* Look for __getitem__ method */
    ViperMethod* getitem_method = vp_method_lookup(obj->class, "__getitem__");
    if (getitem_method && getitem_method->function_ptr) {
        int64_t args[1] = {index};
        return vp_method_call(obj, getitem_method, args);
    }
    
    return 0;
}

/**
 * Set item in container
 */
void vp_object_setitem(ViperInstance* obj, int64_t index, int64_t value) {
    if (!obj) return;
    
    /* Look for __setitem__ method */
    ViperMethod* setitem_method = vp_method_lookup(obj->class, "__setitem__");
    if (setitem_method && setitem_method->function_ptr) {
        int64_t args[2] = {index, value};
        vp_method_call(obj, setitem_method, args);
    }
}

/* ============================================ */
/* Iteration Support                            */
/* ============================================ */

/**
 * Get iterator from an object
 */
ViperInstance* vp_object_iter(ViperInstance* obj) {
    if (!obj) return NULL;
    
    /* Look for __iter__ method */
    ViperMethod* iter_method = vp_method_lookup(obj->class, "__iter__");
    if (iter_method && iter_method->function_ptr) {
        int64_t args[1] = {0};
        int64_t result = vp_method_call(obj, iter_method, args);
        return (ViperInstance*)result;
    }
    
    return NULL;
}

/**
 * Get next item from iterator
 */
int64_t vp_object_next(ViperInstance* iterator, bool* done) {
    if (!iterator) {
        if (done) *done = true;
        return 0;
    }
    
    /* Look for __next__ method */
    ViperMethod* next_method = vp_method_lookup(iterator->class, "__next__");
    if (next_method && next_method->function_ptr) {
        int64_t args[1] = {0};
        int64_t result = vp_method_call(iterator, next_method, args);
        if (done) *done = (result == 0);
        return result;
    }
    
    if (done) *done = true;
    return 0;
}

/* ============================================ */
/* Call Support (Callable Objects)              */
/* ============================================ */

/**
 * Call a callable object
 */
int64_t vp_object_call(ViperInstance* obj, int arg_count, ...) {
    if (!obj) return 0;
    
    /* Look for __call__ method */
    ViperMethod* call_method = vp_method_lookup(obj->class, "__call__");
    if (call_method && call_method->function_ptr) {
        int64_t args[16];  /* Max 16 arguments */
        if (arg_count > 16) {
            arg_count = 16;
        }
        
        va_list ap;
        va_start(ap, arg_count);
        for (int i = 0; i < arg_count; i++) {
            args[i] = va_arg(ap, int64_t);
        }
        va_end(ap);
        
        return vp_method_call(obj, call_method, args);
    }
    
    return 0;
}

/* ============================================ */
/* Hash Support                                 */
/* ============================================ */

/**
 * Get hash of an object
 */
int64_t vp_object_hash(ViperInstance* obj) {
    if (!obj) return 0;
    
    /* Look for __hash__ method */
    ViperMethod* hash_method = vp_method_lookup(obj->class, "__hash__");
    if (hash_method && hash_method->function_ptr) {
        int64_t args[1] = {0};
        return vp_method_call(obj, hash_method, args);
    }
    
    /* Default: hash by pointer value */
    return (int64_t)obj;
}

/* ============================================ */
/* Initialization                               */
/* ============================================ */

/**
 * Initialize an object (call __init__)
 */
void vp_object_init(ViperInstance* obj, int arg_count, ...) {
    if (!obj) return;
    
    /* Look for __init__ method */
    ViperMethod* init_method = vp_method_lookup(obj->class, "__init__");
    if (init_method && init_method->function_ptr) {
        int64_t args[16];  /* Max 16 arguments */
        if (arg_count > 16) {
            arg_count = 16;
        }
        
        va_list ap;
        va_start(ap, arg_count);
        for (int i = 0; i < arg_count; i++) {
            args[i] = va_arg(ap, int64_t);
        }
        va_end(ap);
        
        vp_method_call(obj, init_method, args);
    }
}
