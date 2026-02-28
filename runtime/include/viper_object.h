/**
 * Viper Object Model - OOP Runtime Support
 * Supports classes, inheritance, virtual method dispatch, and properties
 */

#ifndef VIPER_OBJECT_H
#define VIPER_OBJECT_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_arc.h"

/* ============================================ */
/* Class Metadata                               */
/* ============================================ */

typedef struct ViperClass ViperClass;
typedef struct ViperMethod ViperMethod;

/* Method flags */
typedef enum {
    VIPER_METHOD_INSTANCE = 0,
    VIPER_METHOD_STATIC = 1,
    VIPER_METHOD_CLASS = 2,
    VIPER_METHOD_PROPERTY = 4,
    VIPER_METHOD_PROPERTY_SETTER = 8,
    VIPER_METHOD_VIRTUAL = 16,
    VIPER_METHOD_ABSTRACT = 32,
} ViperMethodFlags;

/* Method descriptor */
typedef struct ViperMethod {
    const char* name;           /* Method name */
    void* function_ptr;         /* Pointer to compiled function */
    ViperMethodFlags flags;     /* Method flags */
    int param_count;            /* Number of parameters (excluding self/cls) */
} ViperMethod;

/* Class metadata */
struct ViperClass {
    const char* name;                   /* Class name */
    ViperClass** bases;                 /* Base classes (NULL-terminated) */
    int base_count;                     /* Number of base classes */
    ViperMethod* methods;               /* Method table */
    int method_count;                   /* Number of methods */
    size_t instance_size;               /* Size of instance data */
    void (*init)(void* self);           /* Instance initializer */
    void (*dealloc)(void* self);        /* Instance destructor */
};

/* ============================================ */
/* Instance Object Layout                       */
/* ============================================ */

typedef struct ViperInstance {
    int64_t ref_count;          /* 0:  Reference count for ARC */
    ViperClass* class;          /* 8:  Class metadata pointer */
    void* vtable;               /* 16: Virtual method table (for fast dispatch) */
    /* Instance fields follow in memory */
} ViperInstance;

#define VIPER_INSTANCE_HEADER_SIZE offsetof(ViperInstance, class)

/* ============================================ */
/* Object Creation and Destruction              */
/* ============================================ */

/**
 * Create a new instance of a class
 * @param cls The class to instantiate
 * @return Pointer to new ViperInstance
 */
static inline ViperInstance* vp_object_new(ViperClass* cls) {
    if (!cls) return NULL;
    
    size_t size = sizeof(ViperInstance) + cls->instance_size;
    ViperInstance* obj = (ViperInstance*)malloc(size);
    if (!obj) return NULL;
    
    memset(obj, 0, size);
    obj->ref_count = 1;
    obj->class = cls;
    obj->vtable = NULL;  /* Will be set up by class init if needed */
    
    /* Call instance initializer if present */
    if (cls->init) {
        cls->init(obj);
    }
    
    return obj;
}

/**
 * Destroy an instance
 * @param obj The instance to destroy
 */
static inline void vp_object_del(ViperInstance* obj) {
    if (!obj) return;
    
    /* Call destructor if present */
    if (obj->class && obj->class->dealloc) {
        obj->class->dealloc(obj);
    }
    
    free(obj);
}

/**
 * Get instance data pointer (fields start here)
 * @param obj The instance
 * @return Pointer to instance data
 */
static inline void* vp_instance_data(ViperInstance* obj) {
    if (!obj) return NULL;
    return (char*)obj + sizeof(ViperInstance);
}

/* ============================================ */
/* Field Access Helpers                         */
/* ============================================ */

/**
 * Get field offset for type-safe field access
 * @param offset Offset in bytes from instance data start
 */
#define VIPER_FIELD_OFFSET(offset) (offset)

/**
 * Get a field value
 * @param obj The instance
 * @param type The C type of the field
 * @param offset Byte offset of field
 */
#define VIPER_GET_FIELD(obj, type, offset) \
    (*(type*)((char*)vp_instance_data(obj) + (offset)))

/**
 * Set a field value
 * @param obj The instance
 * @param type The C type of the field
 * @param offset Byte offset of field
 * @param value The value to set
 */
#define VIPER_SET_FIELD(obj, type, offset, value) \
    do { \
        ViperInstance* _obj = (obj); \
        if (_obj) { \
            *(type*)((char*)vp_instance_data(_obj) + (offset)) = (value); \
        } \
    } while(0)

/**
 * Get a field that is a Viper value (with ref counting)
 */
#define VIPER_GET_FIELD_VALUE(obj, type, offset) \
    VIPER_GET_FIELD(obj, type, offset)

/**
 * Set a field that is a Viper value (with ref counting)
 */
#define VIPER_SET_FIELD_VALUE(obj, type, offset, value) \
    do { \
        ViperInstance* _obj = (obj); \
        if (_obj) { \
            type* _field = (type*)((char*)vp_instance_data(_obj) + (offset)); \
            /* Release old value if it's a reference type */ \
            if (*_field) { \
                vp_release((ViperInstance*)(*_field)); \
            } \
            *_field = (value); \
            /* Retain new value */ \
            if (*_field) { \
                vp_retain((ViperInstance*)(*_field)); \
            } \
        } \
    } while(0)

/* ============================================ */
/* Method Dispatch                              */
/* ============================================ */

/**
 * Look up a method by name in a class
 * @param cls The class
 * @param name The method name
 * @return Pointer to method descriptor, or NULL if not found
 */
static inline ViperMethod* vp_method_lookup(ViperClass* cls, const char* name) {
    if (!cls || !name) return NULL;
    
    /* Search in this class */
    for (int i = 0; i < cls->method_count; i++) {
        if (strcmp(cls->methods[i].name, name) == 0) {
            return &cls->methods[i];
        }
    }
    
    /* Search in base classes (MRO - Method Resolution Order) */
    for (int i = 0; i < cls->base_count; i++) {
        ViperMethod* method = vp_method_lookup(cls->bases[i], name);
        if (method) {
            return method;
        }
    }
    
    return NULL;
}

/**
 * Call an instance method
 * @param obj The instance (self)
 * @param method The method descriptor
 * @param args Array of argument values
 * @return Method return value
 */
typedef int64_t (*ViperMethodFunc)(ViperInstance*, int64_t*);

static inline int64_t vp_method_call(ViperInstance* obj, ViperMethod* method, int64_t* args) {
    if (!obj || !method || !method->function_ptr) {
        return 0;
    }
    
    ViperMethodFunc func = (ViperMethodFunc)method->function_ptr;
    return func(obj, args);
}

/**
 * Call a static method
 * @param method The method descriptor
 * @param args Array of argument values
 * @return Method return value
 */
typedef int64_t (*ViperStaticMethodFunc)(int64_t*);

static inline int64_t vp_static_method_call(ViperMethod* method, int64_t* args) {
    if (!method || !method->function_ptr) {
        return 0;
    }
    
    ViperStaticMethodFunc func = (ViperStaticMethodFunc)method->function_ptr;
    return func(args);
}

/**
 * Call a class method
 * @param cls The class (cls)
 * @param method The method descriptor
 * @param args Array of argument values
 * @return Method return value
 */
typedef int64_t (*ViperClassMethodFunc)(ViperClass*, int64_t*);

static inline int64_t vp_class_method_call(ViperClass* cls, ViperMethod* method, int64_t* args) {
    if (!cls || !method || !method->function_ptr) {
        return 0;
    }
    
    ViperClassMethodFunc func = (ViperClassMethodFunc)method->function_ptr;
    return func(cls, args);
}

/* ============================================ */
/* Type Checking and Casting                    */
/* ============================================ */

/**
 * Check if an object is an instance of a class
 * @param obj The instance
 * @param cls The class to check against
 * @return true if obj is an instance of cls or its subclass
 */
static inline bool vp_isinstance(ViperInstance* obj, ViperClass* cls) {
    if (!obj || !cls) return false;
    
    ViperClass* current = obj->class;
    while (current) {
        if (current == cls) {
            return true;
        }
        /* Move to first base class */
        if (current->base_count > 0) {
            current = current->bases[0];
        } else {
            break;
        }
    }
    
    return false;
}

/**
 * Check if a class is a subclass of another
 * @param cls The class to check
 * @param parent The potential parent class
 * @return true if cls is a subclass of parent
 */
static inline bool vp_issubclass(ViperClass* cls, ViperClass* parent) {
    if (!cls || !parent) return false;
    if (cls == parent) return true;
    
    for (int i = 0; i < cls->base_count; i++) {
        if (vp_issubclass(cls->bases[i], parent)) {
            return true;
        }
    }
    
    return false;
}

/**
 * Get the class of an object
 * @param obj The instance
 * @return The class metadata
 */
static inline ViperClass* vp_object_class(ViperInstance* obj) {
    return obj ? obj->class : NULL;
}

/**
 * Get the name of a class
 * @param cls The class
 * @return The class name string
 */
static inline const char* vp_class_name(ViperClass* cls) {
    return cls ? cls->name : "<unknown>";
}

/* ============================================ */
/* Property Support                             */
/* ============================================ */

/**
 * Property getter function type
 */
typedef int64_t (*ViperPropertyGetter)(ViperInstance*);

/**
 * Property setter function type
 */
typedef void (*ViperPropertySetter)(ViperInstance*, int64_t);

/**
 * Property descriptor
 */
typedef struct {
    const char* name;
    ViperPropertyGetter getter;
    ViperPropertySetter setter;
} ViperProperty;

/**
 * Get a property value
 * @param obj The instance
 * @param prop The property descriptor
 * @return Property value
 */
static inline int64_t vp_property_get(ViperInstance* obj, ViperProperty* prop) {
    if (!obj || !prop || !prop->getter) {
        return 0;
    }
    return prop->getter(obj);
}

/**
 * Set a property value
 * @param obj The instance
 * @param prop The property descriptor
 * @param value The value to set
 */
static inline void vp_property_set(ViperInstance* obj, ViperProperty* prop, int64_t value) {
    if (!obj || !prop || !prop->setter) {
        return;
    }
    prop->setter(obj, value);
}

/* ============================================ */
/* Super Call Support                           */
/* ============================================ */

/**
 * Call a method on the parent class
 * @param obj The instance
 * @param method_name The method name
 * @param args Array of argument values
 * @return Method return value
 */
static inline int64_t vp_super_call(ViperInstance* obj, const char* method_name, int64_t* args) {
    if (!obj || !obj->class || !method_name) {
        return 0;
    }
    
    /* Get the parent class */
    if (obj->class->base_count == 0) {
        return 0;  /* No parent class */
    }
    
    ViperClass* parent = obj->class->bases[0];
    ViperMethod* method = vp_method_lookup(parent, method_name);
    
    if (!method) {
        return 0;  /* Method not found in parent */
    }
    
    return vp_method_call(obj, method, args);
}

/* ============================================ */
/* String Representation                        */
/* ============================================ */

/**
 * Get string representation of an object
 * @param obj The instance
 * @return C string (caller must not free)
 */
static inline const char* vp_object_str(ViperInstance* obj) {
    if (!obj) return "<None>";
    
    /* Look for __str__ method */
    ViperMethod* str_method = vp_method_lookup(obj->class, "__str__");
    if (str_method && str_method->function_ptr) {
        int64_t args[1] = {0};
        int64_t result = vp_method_call(obj, str_method, args);
        return (const char*)result;
    }
    
    /* Default representation */
    static char buf[64];
    snprintf(buf, sizeof(buf), "<%s object at %p>", obj->class->name, (void*)obj);
    return buf;
}

/* ============================================ */
/* Equality Comparison                          */
/* ============================================ */

/**
 * Compare two objects for equality
 * @param a First object
 * @param b Second object
 * @return true if equal
 */
static inline bool vp_object_eq(ViperInstance* a, ViperInstance* b) {
    if (!a && !b) return true;
    if (!a || !b) return false;
    if (a == b) return true;
    
    /* Look for __eq__ method */
    ViperMethod* eq_method = vp_method_lookup(a->class, "__eq__");
    if (eq_method && eq_method->function_ptr) {
        int64_t args[1] = {(int64_t)b};
        int64_t result = vp_method_call(a, eq_method, args);
        return result != 0;
    }
    
    /* Default: compare pointers */
    return a == b;
}

/* ============================================ */
/* Length Support (for containers)              */
/* ============================================ */

/**
 * Get length of an object (if it supports __len__)
 * @param obj The instance
 * @return Length, or -1 if not supported
 */
static inline int64_t vp_object_len(ViperInstance* obj) {
    if (!obj) return -1;
    
    /* Look for __len__ method */
    ViperMethod* len_method = vp_method_lookup(obj->class, "__len__");
    if (len_method && len_method->function_ptr) {
        int64_t args[1] = {0};
        return vp_method_call(obj, len_method, args);
    }
    
    return -1;  /* Not supported */
}

#endif /* VIPER_OBJECT_H */
