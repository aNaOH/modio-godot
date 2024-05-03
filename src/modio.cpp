#include "modio.h"
#include "modio/ModioSDK.h"
#include <godot_cpp/core/class_db.hpp>
#include <string>

using namespace godot;

void ModIO::_bind_methods() {
    ClassDB::bind_method(D_METHOD("init", "api_key", "game_id"), &ModIO::init);
}

ModIO::ModIO() {
	
}

ModIO::~ModIO() {
	// Add your cleanup here.
}

bool ModIO::init(String key, unsigned int game){

	std::string keyFormated = key.utf8().get_data();
	Modio::Optional<bool> SDKInitialized;

	Modio::InitializeOptions Options;
	Options.APIKey = Modio::ApiKey(keyFormated);
	Options.GameID = Modio::GameID(game);
	Options.User = "LocalProfileName";
	Options.PortalInUse = Modio::Portal::None;
	Options.GameEnvironment = Modio::Environment::Live;

	Modio::InitializeAsync(Options, [&SDKInitialized](Modio::ErrorCode ec) {
		if (ec)
		{
			return false;
		}
		else
		{
			return true;
		}
	});
}

void ModIO::_process(double delta) {
	Modio::RunPendingHandlers();
}