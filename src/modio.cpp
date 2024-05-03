#include "modio.h"
#include "modio/ModioSDK.h"
#include <godot_cpp/core/class_db.hpp>
#include <string>

using namespace godot;

void ModIO::_bind_methods() {
    ClassDB::bind_method(D_METHOD("init", "api_key", "game_id"), &ModIO::init);

	ADD_SIGNAL(MethodInfo("on_init", PropertyInfo(Variant::BOOL, "ok")));
}

ModIO::ModIO() {
	
}

ModIO::~ModIO() {
	// Add your cleanup here.
}

void ModIO::init(String key, unsigned int game){

	std::string keyFormated = key.utf8().get_data();
	Modio::Optional<bool> SDKInitialized;

	Modio::InitializeOptions Options;
	Options.APIKey = Modio::ApiKey(keyFormated);
	Options.GameID = Modio::GameID(game);
	Options.User = "LocalProfileName";
	Options.PortalInUse = Modio::Portal::None;
	Options.GameEnvironment = Modio::Environment::Live;

	Modio::InitializeAsync(Options, [&SDKInitialized, this](Modio::ErrorCode ec) {
		if (ec)
		{
			emit_signal("on_init", false);
		}
		else
		{
			emit_signal("on_init", true);
		}
	});
}

void ModIO::_process(double delta) {
	Modio::RunPendingHandlers();
}