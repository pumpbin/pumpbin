@0x96d5aac4519892f3;

struct Plugin {
	version @0 : Text;
	info @1 :PluginInfo;
	replace @2 :PluginReplace;
	bins @3 :PluginBins;
	plugins @4 :PluginPlugins;

	struct PluginInfo{
		pluginName @0 :Text;
		author @1 :Text;
		version @2 :Text;
		desc @3 :Text;
	}

	struct PluginReplace {
		srcPrefix @0 :Data;
		sizeHolder @1 :Data;
		maxLen @2 :UInt64;
	}

	struct PluginBins {
		windows @0 :Bins;
		linux @1 :Bins;
		darwin @2 :Bins;

		struct Bins {
			executable @0 :Data;
			dynamicLibrary @1 :Data;
		}
	}

	struct PluginPlugins {
		encryptShellcode @0 :Data;
		formatEncryptedShellcode @1 :Data;
		formatUrlRemote @2 :Data;
		uploadFinalShellcodeRemote @3 :Data;
	}
}
