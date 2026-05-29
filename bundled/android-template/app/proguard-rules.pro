# Add project specific ProGuard rules here.
-keepattributes *Annotation*
-keepattributes Exceptions,InnerClasses,Signature,Deprecated,SourceFile,LineNumberTable,*Annotation*,EnclosingMethod
-dontwarn javax.annotation.**
-dontwarn kotlin.Unit

-keep class io.dcloud.** { *; }
-dontwarn io.dcloud.**

-keep public class * extends android.app.Activity
-keep public class * extends android.app.Application
-keep public class * extends android.app.Service
-keep public class * extends android.content.BroadcastReceiver
-keep public class * extends android.content.ContentProvider

-keepclassmembers enum * {
    public static **[] values();
    public static ** valueOf(java.lang.String);
}

-keep class com.alibaba.fastjson.** { *; }
-dontwarn com.alibaba.fastjson.**

-keep class net.lingala.zip4j.** { *; }
-dontwarn net.lingala.zip4j.**

-keep class com.facebook.** { *; }
-dontwarn com.facebook.**

-keep class pl.droidsonroids.gif.** { *; }
-dontwarn pl.droidsonroids.gif.**

-keep class com.getkeepsafe.relinker.** { *; }
-dontwarn com.getkeepsafe.relinker.**

-keep class com.bumptech.glide.** { *; }
-dontwarn com.bumptech.glide.**

-keep class okhttp3.** { *; }
-dontwarn okhttp3.**
-keep interface okhttp3.** { *; }

-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile
