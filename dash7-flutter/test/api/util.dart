import 'dart:io';

import 'package:dash7/main.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';

ExternalLibrary useLibrary() {
  const libName = 'dash7_dart';
  final libPrefix = {
    Platform.isWindows: '',
    Platform.isMacOS: 'lib',
    Platform.isLinux: 'lib',
  }[true]!;
  final libSuffix = {
    Platform.isWindows: 'dll',
    Platform.isMacOS: 'dylib',
    Platform.isLinux: 'so',
  }[true]!;
  final dylibPath = '../target/debug/$libPrefix$libName.$libSuffix';
  return ExternalLibrary.open(dylibPath);
}

Future<void> init({bool skipLibInit = false}) async {
  if (!skipLibInit) {
    await dash7.init(externalLibrary: useLibrary());
  }
}
