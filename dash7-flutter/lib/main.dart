// Dart API related
import 'src/frb_generated.dart';

export 'src/api/varint.dart';
// Rust FFI related
export 'src/frb_generated.dart' show dash7;

Future<void> initdash7() async {
  await dash7.init();
}

// const String _libName = 'dash7';

// /// The dynamic library in which the symbols for [dash7Bindings] can be found.
// final DynamicLibrary _dylib = () {
//   if (Platform.isMacOS || Platform.isIOS) {
//     return DynamicLibrary.open('$_libName.framework/$_libName');
//   }
//   if (Platform.isAndroid || Platform.isLinux) {
//     return DynamicLibrary.open('lib$_libName.so');
//   }
//   if (Platform.isWindows) {
//     return DynamicLibrary.open('$_libName.dll');
//   }
//   throw UnsupportedError('Unknown platform: ${Platform.operatingSystem}');
// }();

// /// The bindings to the native functions in [_dylib].
// final dash7Bindings _bindings = dash7Bindings(_dylib);



