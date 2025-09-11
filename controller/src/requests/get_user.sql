select 
	u."name" as UserName,
	r."name" as Role
from 
	userrole u2  
left join roles r  
		on r.id = u2.role_id  
left join users u 
	on u.id = u2.user_id