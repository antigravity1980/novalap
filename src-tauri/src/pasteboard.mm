#import <Cocoa/Cocoa.h>

extern "C" {

const char* lap_get_drag_image_url(void) {
    @autoreleasepool {
        NSPasteboard *pb = [NSPasteboard pasteboardWithName:NSPasteboardNameDrag];
        if (!pb) return NULL;

        NSArray<NSURL *> *urls = [pb readObjectsForClasses:@[[NSURL class]] options:nil];
        for (NSURL *url in urls) {
            NSString *str = [url absoluteString];
            if (str && ([str hasPrefix:@"http://"] || [str hasPrefix:@"https://"])) {
                return strdup([str UTF8String]);
            }
        }
        return NULL;
    }
}

const char* lap_read_clipboard_files(void) {
    @autoreleasepool {
        NSPasteboard *pb = [NSPasteboard generalPasteboard];
        if (!pb) return NULL;
        
        NSArray *classes = @[[NSURL class]];
        NSDictionary *options = @{NSPasteboardURLReadingFileURLsOnlyKey: @YES};
        NSArray<NSURL *> *urls = [pb readObjectsForClasses:classes options:options];
        if (!urls || [urls count] == 0) return NULL;
        
        NSMutableString *result = [NSMutableString string];
        for (NSURL *url in urls) {
            NSString *path = [url path];
            if (path) {
                [result appendFormat:@"%@\n", path];
            }
        }
        if ([result length] == 0) return NULL;
        return strdup([result UTF8String]);
    }
}

void lap_write_clipboard_files(const char* paths_str) {
    @autoreleasepool {
        NSPasteboard *pb = [NSPasteboard generalPasteboard];
        if (!pb) return;
        
        [pb clearContents];
        
        NSString *str = [NSString stringWithUTF8String:paths_str];
        NSArray<NSString *> *paths = [str componentsSeparatedByString:@"\n"];
        NSMutableArray<NSURL *> *urls = [NSMutableArray array];
        
        for (NSString *path in paths) {
            if ([path length] == 0) continue;
            NSURL *url = [NSURL fileURLWithPath:path];
            if (url) {
                [urls addObject:url];
            }
        }
        
        if ([urls count] > 0) {
            [pb writeObjects:urls];
        }
    }
}

void lap_free_string(const char* ptr) {
    if (ptr) free((void*)ptr);
}

}
